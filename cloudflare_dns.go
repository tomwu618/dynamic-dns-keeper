package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"time"
)

// CloudflareUpdater handles DNS updates for Cloudflare
type CloudflareUpdater struct {
	apiKey      string
	email       string
	zoneID      string
	recordType  string // "A" or "AAAA"
	recordName  string // subdomain part, e.g., "www"
	domain      string // main domain, e.g., "example.com"
	ttl         int    // TTL in seconds, 1 for auto
	proxied     bool
	client      *http.Client
	currentIP   string // Store the last known IP on Cloudflare
	dnsRecordID string // Store the ID of the DNS record to update
}

// CloudflareListResponse is for listing DNS records
type CloudflareListResponse struct {
	Success bool `json:"success"`
	Errors  []struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
	} `json:"errors"`
	Messages []interface{} `json:"messages"`
	Result   []struct {
		ID         string    `json:"id"`
		Type       string    `json:"type"`
		Name       string    `json:"name"`
		Content    string    `json:"content"`
		Proxiable  bool      `json:"proxiable"`
		Proxied    bool      `json:"proxied"`
		TTL        int       `json:"ttl"`
		Locked     bool      `json:"locked"`
		ZoneID     string    `json:"zone_id"`
		ZoneName   string    `json:"zone_name"`
		CreatedOn  time.Time `json:"created_on"`
		ModifiedOn time.Time `json:"modified_on"`
		Data       struct{}  `json:"data"` // Assuming empty or not needed for A/AAAA
	} `json:"result"`
	ResultInfo struct {
		Page       int `json:"page"`
		PerPage    int `json:"per_page"`
		Count      int `json:"count"`
		TotalCount int `json:"total_count"`
		TotalPages int `json:"total_pages"`
	} `json:"result_info"`
}

// CloudflareUpdateResponse is for the update call
type CloudflareUpdateResponse struct {
	Success bool `json:"success"`
	Errors  []struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
	} `json:"errors"`
	Result struct {
		ID      string `json:"id"`
		Content string `json:"content"`
		// ... other fields similar to list result
	} `json:"result"`
}

// NewCloudflareUpdater creates a new Cloudflare DNS updater instance
func NewCloudflareUpdater(params map[string]interface{}) (*CloudflareUpdater, error) {
	apiKey, err := getStringParam(params, "api_key", true)
	if err != nil {
		return nil, err
	}
	email, err := getStringParam(params, "email", true)
	if err != nil {
		return nil, err
	}
	zoneID, err := getStringParam(params, "zone_id", true)
	if err != nil {
		return nil, err
	}
	recordType, err := getStringParam(params, "record_type", true)
	if err != nil {
		return nil, err
	}
	recordName, err := getStringParam(params, "record_name", true) // e.g. "subdomain"
	if err != nil {
		return nil, err
	}
	domain, err := getStringParam(params, "domain", true) // e.g. "example.com"
	if err != nil {
		return nil, err
	}

	ttlInt64, err := getInt64Param(params, "record_ttl", true)
	if err != nil {
		return nil, fmt.Errorf("invalid cloudflare record_ttl: %w", err)
	}
	ttl := int(ttlInt64) // Cloudflare API often uses int for TTL

	proxied, err := getBoolParam(params, "record_proxied", true)
	if err != nil {
		return nil, fmt.Errorf("invalid cloudflare record_proxied: %w", err)
	}

	updater := &CloudflareUpdater{
		apiKey:     apiKey,
		email:      email,
		zoneID:     zoneID,
		recordType: recordType,
		recordName: recordName, // This is the 'name' in API, which is subdomain part
		domain:     domain,     // This is used to form the full name for query if needed
		ttl:        ttl,
		proxied:    proxied,
		client:     &http.Client{Timeout: 10 * time.Second},
	}

	// Fetch initial record ID and IP
	err = updater.fetchDNSRecordDetails()
	if err != nil {
		log.Printf("Cloudflare: Failed to fetch initial DNS record details for %s.%s: %v\n", recordName, domain, err)
		// Depending on policy, might return error or proceed (update will then try to create or fail)
		// For now, let's proceed, Update will handle it.
	}

	return updater, nil
}

func (u *CloudflareUpdater) fetchDNSRecordDetails() error {
	// The 'name' parameter for Cloudflare API is the FQDN (e.g., subdomain.example.com)
	fqdn := fmt.Sprintf("%s.%s", u.recordName, u.domain)
	if u.recordName == "@" || u.recordName == u.domain { // Handle root domain case
		fqdn = u.domain
	}

	url := fmt.Sprintf("https://api.cloudflare.com/client/v4/zones/%s/dns_records?type=%s&name=%s",
		u.zoneID, u.recordType, fqdn)

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return fmt.Errorf("failed to create Cloudflare GET request: %w", err)
	}
	req.Header.Set("X-Auth-Email", u.email)
	req.Header.Set("X-Auth-Key", u.apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := u.client.Do(req)
	if err != nil {
		return fmt.Errorf("Cloudflare API GET request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read Cloudflare API response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("Cloudflare API GET request returned status %d: %s", resp.StatusCode, string(body))
	}

	var listResp CloudflareListResponse
	if err := json.Unmarshal(body, &listResp); err != nil {
		return fmt.Errorf("failed to unmarshal Cloudflare API response: %w. Body: %s", err, string(body))
	}

	if !listResp.Success {
		return fmt.Errorf("Cloudflare API GET request was not successful: %v", listResp.Errors)
	}

	if len(listResp.Result) == 0 {
		log.Printf("Cloudflare: No %s record found for %s. Will attempt to create if update is called.", u.recordType, fqdn)
		u.dnsRecordID = "" // Explicitly mark as no existing record found
		u.currentIP = ""
		return nil // Not an error, record just doesn't exist
	}

	// Assuming the first record is the one we want if multiple are returned (should be specific enough)
	record := listResp.Result[0]
	u.dnsRecordID = record.ID
	u.currentIP = record.Content
	log.Printf("Cloudflare: Fetched existing record for %s. ID: %s, IP: %s\n", fqdn, u.dnsRecordID, u.currentIP)
	return nil
}

// Update updates the DNS record if the IP address has changed
func (u *CloudflareUpdater) Update(newIP net.IP) (bool, error) {
	newIPStr := newIP.String()

	// Re-fetch details in case they changed or wasn't found initially
	// Or rely on initially fetched details if confident. For robustness, re-fetch or verify.
	// For simplicity here, assume initial fetch was sufficient or we're creating.
	// A more robust implementation would call fetchDNSRecordDetails() here or have a more complex state.
	// Let's call it again to ensure we have the latest state.
	err := u.fetchDNSRecordDetails()
	if err != nil {
		log.Printf("Cloudflare: Error re-fetching DNS record details for %s.%s before update: %v\n", u.recordName, u.domain, err)
		// Decide if to proceed or not. Let's proceed, as we might be creating the record.
	}

	if u.dnsRecordID != "" && u.currentIP == newIPStr {
		log.Printf("Cloudflare: IP for %s.%s (%s) is already up to date.\n", u.recordName, u.domain, newIPStr)
		return false, nil
	}

	fqdn := fmt.Sprintf("%s.%s", u.recordName, u.domain)
	if u.recordName == "@" || u.recordName == u.domain {
		fqdn = u.domain
	}

	log.Printf("Cloudflare: Updating IP for %s from %s to %s\n", fqdn, u.currentIP, newIPStr)

	payload := map[string]interface{}{
		"type":    u.recordType,
		"name":    fqdn, // Cloudflare uses FQDN for 'name' in PUT/POST
		"content": newIPStr,
		"ttl":     u.ttl,
		"proxied": u.proxied,
	}

	jsonPayload, err := json.Marshal(payload)
	if err != nil {
		return false, fmt.Errorf("failed to marshal Cloudflare update payload: %w", err)
	}

	var req *http.Request
	var url string

	if u.dnsRecordID == "" { // Record does not exist, create it
		log.Printf("Cloudflare: Record %s does not exist. Attempting to create.\n", fqdn)
		url = fmt.Sprintf("https://api.cloudflare.com/client/v4/zones/%s/dns_records", u.zoneID)
		req, err = http.NewRequest("POST", url, bytes.NewBuffer(jsonPayload))
	} else { // Record exists, update it
		url = fmt.Sprintf("https://api.cloudflare.com/client/v4/zones/%s/dns_records/%s", u.zoneID, u.dnsRecordID)
		req, err = http.NewRequest("PUT", url, bytes.NewBuffer(jsonPayload))
	}

	if err != nil {
		return false, fmt.Errorf("failed to create Cloudflare update/create request: %w", err)
	}

	req.Header.Set("X-Auth-Email", u.email)
	req.Header.Set("X-Auth-Key", u.apiKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := u.client.Do(req)
	if err != nil {
		return false, fmt.Errorf("Cloudflare API update/create request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return false, fmt.Errorf("failed to read Cloudflare API update/create response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return false, fmt.Errorf("Cloudflare API update/create request returned status %d: %s", resp.StatusCode, string(body))
	}

	var updateResp CloudflareUpdateResponse
	if err := json.Unmarshal(body, &updateResp); err != nil {
		return false, fmt.Errorf("failed to unmarshal Cloudflare API update/create response: %w. Body: %s", err, string(body))
	}

	if !updateResp.Success {
		return false, fmt.Errorf("Cloudflare API update/create was not successful: %v", updateResp.Errors)
	}

	// Update local state
	u.currentIP = updateResp.Result.Content // Or newIPStr
	if u.dnsRecordID == "" {                // If created, store new ID
		u.dnsRecordID = updateResp.Result.ID
		log.Printf("Cloudflare: Successfully CREATED DNS record %s. ID: %s, IP: %s\n", fqdn, u.dnsRecordID, u.currentIP)
	} else {
		log.Printf("Cloudflare: Successfully UPDATED DNS record %s. ID: %s, IP: %s\n", fqdn, u.dnsRecordID, u.currentIP)
	}

	return true, nil
}
