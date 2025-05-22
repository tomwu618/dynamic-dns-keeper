package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"
)

const defaultConfigPath = "/etc/ddk/config.toml" // Or for Windows: "config.toml" in current dir

func main() {
	// Command line flag for config file path
	configPath := flag.String("config", defaultConfigPath, "Path to the TOML configuration file.")
	// Allow shorthand -c
	cFlag := flag.String("c", defaultConfigPath, "Path to the TOML configuration file (shorthand).")
	flag.Parse()

	actualConfigPath := *configPath
	// If -c was used and -config was not, -c takes precedence (or rather, check if it differs from default)
	// A better way for flag overriding: check if the flag was actually set by user.
	// For simplicity, if -c is different from default and -config is default, use -c.
	userSetC := false
	flag.Visit(func(f *flag.Flag) {
		if f.Name == "c" {
			userSetC = true
		}
	})
	if userSetC && *configPath == defaultConfigPath { // if -c used and -config is default
		actualConfigPath = *cFlag
	} else if *configPath == defaultConfigPath && *cFlag != defaultConfigPath { // if -config is default and -c is not default
		actualConfigPath = *cFlag
	}

	// Environment variable DDK_CONFIG takes precedence
	envConfigPath := os.Getenv("DDK_CONFIG")
	if envConfigPath != "" {
		actualConfigPath = envConfigPath
	}

	cfg, err := ReadConfig(actualConfigPath)
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	log.Printf("Loaded configuration: API Version %s, PostUpWait %ds, CheckInterval %ds\n",
		cfg.Global.APIVersion, cfg.Global.PostUpWait, cfg.Global.CheckInterval)

	// Initial "post_up" sequence
	if cfg.Global.PostUpWait > 0 {
		log.Printf("Waiting for network... (%d seconds)\n", cfg.Global.PostUpWait)
		time.Sleep(time.Duration(cfg.Global.PostUpWait) * time.Second)
	}
	log.Println("Start")
	if cfg.Global.PostUpCmd != "" {
		RunCommandArray(cfg.Global.PostUpCmd)
	}

	var wg sync.WaitGroup
	for _, record := range cfg.Records {
		wg.Add(1)
		// Pass record by value to avoid closure capturing the loop variable
		go ProcessRecord(record, cfg.Global, &wg)
	}

	// Wait for all workers to finish (they run indefinitely, so this is for graceful shutdown)
	// For a daemon, we'd usually select{} here and handle signals.

	// Set up signal handling for graceful shutdown
	sigs := make(chan os.Signal, 1)
	done := make(chan bool, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		sig := <-sigs
		fmt.Println()
		log.Printf("Received signal: %s. Shutting down workers...\n", sig)
		// Here you would signal all goroutines to stop if they supported it (e.g. via a quit channel)
		// Since ProcessRecord runs an infinite loop with a ticker,
		// a quit channel would be the proper way. For now, we'll just exit.
		// For a true graceful shutdown, each worker would need to listen on a shared quit channel.
		done <- true
	}()

	log.Println("DDNS service started. Press Ctrl+C to exit.")
	// If workers are truly daemon-like and `wg.Wait()` isn't desired for the main flow:
	// select {} // Block forever until signal

	// If you want main to exit only when all workers (hypothetically) could finish:
	// wg.Wait() // This will block indefinitely as workers are infinite loops

	// Using the signal handling:
	<-done
	log.Println("DDNS service stopped.")

}
