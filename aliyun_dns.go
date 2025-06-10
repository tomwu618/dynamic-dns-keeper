package main

import (
	"fmt"
	"log"
	"net"

	alidns20150109 "github.com/alibabacloud-go/alidns-20150109/v4/client"
	openapi "github.com/alibabacloud-go/darabonba-openapi/v2/client"
	"github.com/alibabacloud-go/tea-utils/v2/service"
	"github.com/alibabacloud-go/tea/tea"
)

// AliyunUpdater handles DNS updates for Aliyun
type AliyunUpdater struct {
	client     *alidns20150109.Client
	DomainName string // e.g., "example.com"
	RecordId   string
	RR         string // e.g., "home-studio" (subdomain part)
	RecordType string // e.g., "A", "AAAA"
	TTL        int64
	Line       string // Optional, e.g., "default", "telecom"
	currentIP  string // Store the last known IP on Aliyun to avoid unnecessary updates
}

// NewAliyunUpdater creates a new Aliyun DNS updater instance
func NewAliyunUpdater(params map[string]interface{}) (*AliyunUpdater, error) {
	accessKeyId, err := getStringParam(params, "key_id", true)
	if err != nil {
		return nil, err
	}

	accessKeySecret, err := getStringParam(params, "key_secret", true)
	if err != nil {
		return nil, err
	}

	recordId, err := getStringParam(params, "record_id", true)
	if err != nil {
		return nil, err
	}

	rr, err := getStringParam(params, "record_rr", true)
	if err != nil {
		return nil, err
	}

	recordType, err := getStringParam(params, "record_type", true)
	if err != nil {
		return nil, err
	}

	domainName, err := getStringParam(params, "domain_name", true) // CRITICAL: Must be in config
	if err != nil {
		return nil, fmt.Errorf("aliyun 'domain_name' is missing in api_param: %w", err)
	}

	ttlStr, err := getStringParam(params, "record_ttl", true) // Assuming TTL is string in config like "600"
	if err != nil {
		return nil, err
	}
	ttl, err := parseInt64(ttlStr) // Helper to parse string to int64
	if err != nil {
		return nil, fmt.Errorf("invalid aliyun record_ttl '%s': %w", ttlStr, err)
	}

	line, _ := getStringParam(params, "record_line", false) // Optional
	if line == "" {
		line = "default" // Default line if not specified
	}

	apiEndpoint, _ := getStringParam(params, "endpoint", false)
	if apiEndpoint == "" {
		apiEndpoint = "alidns.aliyuncs.com" // Default Aliyun endpoint
	}

	config := &openapi.Config{
		AccessKeyId:     tea.String(accessKeyId),
		AccessKeySecret: tea.String(accessKeySecret),
		Endpoint:        tea.String(apiEndpoint),
	}

	client, err := alidns20150109.NewClient(config)
	if err != nil {
		return nil, fmt.Errorf("failed to create Aliyun client: %w", err)
	}

	updater := &AliyunUpdater{
		client:     client,
		DomainName: domainName,
		RecordId:   recordId,
		RR:         rr,
		RecordType: recordType,
		TTL:        ttl,
		Line:       line,
	}
	return updater, nil
}

// fetchCurrentRecordIP fetches the current IP address of the DNS record from Aliyun.
// This helps in avoiding unnecessary updates if the IP hasn't changed.
func (u *AliyunUpdater) fetchCurrentRecordIP() (string, error) {
	// *** MODIFICATION START ***
	// Set explicit timeouts for the API call to prevent long hangs
	runtime := &service.RuntimeOptions{
		ConnectTimeout: tea.Int(5000),  // 5 seconds connect timeout
		ReadTimeout:    tea.Int(10000), // 10 seconds read timeout
	}
	// *** MODIFICATION END ***

	// NOTE: The original code had a call to DescribeSubDomainRecords followed by DescribeDomainRecordInfo.
	// Using DescribeDomainRecordInfo is more direct and efficient if we already have the RecordId.
	// The log showed an error with DescribeSubDomainRecords, but for simplicity and efficiency,
	// we will rely on the more direct DescribeDomainRecordInfo and apply the timeout to it.
	infoRequest := &alidns20150109.DescribeDomainRecordInfoRequest{
		RecordId: tea.String(u.RecordId),
	}
	infoResponse, err := u.client.DescribeDomainRecordInfoWithOptions(infoRequest, runtime)
	if err != nil {
		return "", fmt.Errorf("aliyun DescribeDomainRecordInfo API error for RecordID %s: %w", u.RecordId, err)
	}

	if infoResponse != nil && infoResponse.Body != nil && infoResponse.Body.Value != nil {
		return tea.StringValue(infoResponse.Body.Value), nil
	}

	return "", fmt.Errorf("could not determine current IP for RecordID %s (Aliyun)", u.RecordId)
}

// Update updates the DNS record if the IP address has changed
func (u *AliyunUpdater) Update(newIP net.IP) (bool, error) {
	newIPStr := newIP.String()

	// Fetch current IP from DNS provider before updating
	// This avoids updating if the IP is already correct or if the local currentIP state is stale.
	remoteIP, err := u.fetchCurrentRecordIP()
	if err != nil {
		// If fetching remote IP fails (e.g., due to timeout), log the error and proceed.
		// The subsequent update attempt will either succeed or fail gracefully.
		log.Printf("Aliyun: Failed to fetch remote IP for %s.%s: %v. Proceeding with update attempt.\n", u.RR, u.DomainName, err)
	} else {
		u.currentIP = remoteIP
		log.Printf("Aliyun: Remote IP for %s.%s is %s\n", u.RR, u.DomainName, u.currentIP)
	}

	if u.currentIP == newIPStr {
		log.Printf("Aliyun: IP for %s.%s (%s) is already up to date.\n", u.RR, u.DomainName, newIPStr)
		return false, nil
	}

	log.Printf("Aliyun: Updating IP for %s.%s from %s to %s\n", u.RR, u.DomainName, u.currentIP, newIPStr)

	request := &alidns20150109.UpdateDomainRecordRequest{
		RecordId: tea.String(u.RecordId),
		RR:       tea.String(u.RR),
		Type:     tea.String(u.RecordType),
		Value:    tea.String(newIPStr),
		TTL:      tea.Int64(u.TTL),
		Line:     tea.String(u.Line),
	}

	// *** MODIFICATION START ***
	// Apply the same timeouts to the update request
	runtime := &service.RuntimeOptions{
		ConnectTimeout: tea.Int(5000),
		ReadTimeout:    tea.Int(10000),
	}
	// *** MODIFICATION END ***

	_, err = u.client.UpdateDomainRecordWithOptions(request, runtime)
	if err != nil {
		if sdkErr, ok := err.(*tea.SDKError); ok {
			errMsg := tea.StringValue(sdkErr.Message)
			log.Printf("Aliyun API UpdateDomainRecord error for %s.%s: Code: %s, Message: %s, Data: %v\n",
				u.RR, u.DomainName, tea.StringValue(sdkErr.Code), errMsg, sdkErr.Data)
			return false, fmt.Errorf("aliyun API error: %s", errMsg)
		}
		log.Printf("Aliyun API UpdateDomainRecord raw error for %s.%s: %v\n", u.RR, u.DomainName, err)
		return false, fmt.Errorf("aliyun update failed: %w", err)
	}

	u.currentIP = newIPStr
	log.Printf("Aliyun: Successfully updated IP for %s.%s to %s\n", u.RR, u.DomainName, newIPStr)
	return true, nil
}
