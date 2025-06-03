package main

import (
	"log"
	"net"
	"strconv"
	"strings"
	"sync"
	"time"
)

// DNSRecordUpdater defines the interface for DNS updaters
type DNSRecordUpdater interface {
	Update(ip net.IP) (bool, error) // Returns true if updated, false otherwise
}

// Global map to track IPs for which on_update_cmd has been run in the current "session"
var (
	updatedIPs      = make(map[string]bool)
	updatedIPsMutex = &sync.Mutex{}
)

// parseInt64 is a helper function to parse a string to int64
func parseInt64(s string) (int64, error) {
	return strconv.ParseInt(s, 10, 64)
}

// ProcessRecord is the worker function for a single DNS record
func ProcessRecord(recordCfg RecordConfig, globalCfg GlobalConfig, wg *sync.WaitGroup) {
	defer wg.Done()

	log.Printf("Starting worker for registrar: %s, cmd: %s\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd)

	var updater DNSRecordUpdater
	var err error

	switch strings.ToLower(recordCfg.DomainRegistrar) {
	case "aliyun":
		updater, err = NewAliyunUpdater(recordCfg.ApiParams)
	case "cloudflare":
		updater, err = NewCloudflareUpdater(recordCfg.ApiParams)
	default:
		log.Printf("Unsupported domain registrar: %s for cmd: %s\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd)
		return
	}

	if err != nil {
		log.Printf("Failed to initialize DNS updater for %s (%s): %v\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, err)
		return
	}

	ticker := time.NewTicker(time.Duration(globalCfg.CheckInterval) * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			log.Printf("Worker for %s (%s): Checking IP...\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd)
			ipStr, err := RunCommand(recordCfg.IpAddressFromCmd)
			if err != nil {
				log.Printf("Worker for %s (%s): Failed to get IP from command '%s': %v\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, recordCfg.IpAddressFromCmd, err)
				continue
			}
			ipStr = strings.TrimSpace(ipStr)
			if ipStr == "" {
				log.Printf("Worker for %s (%s): Command '%s' returned empty IP.\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, recordCfg.IpAddressFromCmd)
				continue
			}

			currentIP := net.ParseIP(ipStr)
			if currentIP == nil {
				log.Printf("Worker for %s (%s): Failed to parse IP address '%s'\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, ipStr)
				continue
			}

			// Determine if record is A or AAAA based on IP type, or from config if specified
			// This logic might be needed if RecordType in ApiParams isn't definitive (e.g. for Cloudflare)
			// For Aliyun, RecordType is explicit.
			// if currentIP.To4() == nil && recordCfg.ApiParams["record_type"] == "A" {
			// 	log.Printf("Warning: Fetched IPv6 for an A record config for %s", recordCfg.IpAddressFromCmd)
			// }
			// if currentIP.To4() != nil && recordCfg.ApiParams["record_type"] == "AAAA" {
			//  log.Printf("Warning: Fetched IPv4 for an AAAA record config for %s", recordCfg.IpAddressFromCmd)
			// }

			updated, err := updater.Update(currentIP)
			if err != nil {
				log.Printf("Worker for %s (%s): DNS update failed: %v\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, err)
				continue // Don't run on_update_cmd if update itself failed
			}

			if updated {
				log.Printf("Worker for %s (%s): DNS record updated successfully to %s.\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, currentIP.String())
				if recordCfg.IpAddressOnUpdateCmd != "" {
					updatedIPsMutex.Lock()
					alreadyProcessed := updatedIPs[ipStr]
					if !alreadyProcessed {
						updatedIPs[ipStr] = true
					}
					updatedIPsMutex.Unlock()

					if !alreadyProcessed {
						cmdToRun := strings.ReplaceAll(recordCfg.IpAddressOnUpdateCmd, "${IP_ADDRESS}", ipStr)
						log.Printf("Worker for %s (%s): Running on_update_cmd: %s\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, cmdToRun)
						RunCommandArray(cmdToRun)
					} else {
						log.Printf("Worker for %s (%s): on_update_cmd for IP %s already processed in this session. Skipping.\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, ipStr)
					}
				}
			} else {
				// log.Printf("Worker for %s (%s): DNS record IP %s was already up-to-date. No update performed.\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd, currentIP.String())
				// This is now logged within the updater methods
			}
			// Add a way to gracefully shut down the worker if needed, e.g. via a channel
			// case <-quitChannel:
			//  log.Printf("Worker for %s (%s) stopping.\n", recordCfg.DomainRegistrar, recordCfg.IpAddressFromCmd)
			//  return
		}
	}
}
