package main

import (
	"fmt"
	"log"
	"os/exec"
	"strings"
)

// RunCommand executes a single command and returns its standard output or an error.
// The command is expected to be a string where the first part is the command
// and subsequent parts are arguments, separated by spaces.
func RunCommand(command string) (string, error) {
	if command == "" {
		return "", fmt.Errorf("command is empty")
	}
	log.Printf("Run %s\n", command)

	parts := strings.Fields(command) // strings.Fields handles multiple spaces correctly
	if len(parts) == 0 {
		return "", fmt.Errorf("command is empty after splitting")
	}

	cmd := exec.Command(parts[0], parts[1:]...)
	output, err := cmd.CombinedOutput() // CombinedOutput gets both stdout and stderr

	if err != nil {
		log.Printf("  Error: %s, Output: %s\n", err, string(output))
		return string(output), err // Return output even on error, as it might contain useful info
	}

	trimmedOutput := strings.TrimSpace(string(output))
	log.Printf("  Success: %s\n", trimmedOutput)
	return trimmedOutput, nil
}

// RunCommandArray executes a series of commands separated by semicolons.
func RunCommandArray(commands string) {
	if commands == "" {
		return
	}
	splitCommands := strings.Split(commands, ";")
	for _, cmdStr := range splitCommands {
		trimmedCmd := strings.TrimSpace(cmdStr)
		if trimmedCmd != "" {
			_, err := RunCommand(trimmedCmd)
			if err != nil {
				log.Printf("Error running command from array '%s': %v\n", trimmedCmd, err)
			}
		}
	}
}
