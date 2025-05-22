package main

import (
	"fmt"
	"io/ioutil"
	"strconv"

	"github.com/BurntSushi/toml"
)

// GlobalConfig 对应 TOML 中的 [global]
type GlobalConfig struct {
	APIVersion    string `toml:"api_version"`
	PostUpWait    int64  `toml:"post_up_wait"` // Changed to int64 for time.Duration
	PostUpCmd     string `toml:"post_up_cmd"`
	CheckInterval int64  `toml:"check_interval_seconds"` // 新增: 检查间隔，单位秒
}

// RecordConfig 对应 TOML 中的 [[record]]
type RecordConfig struct {
	DomainRegistrar      string                 `toml:"domain_registrar"`
	IpAddressFromCmd     string                 `toml:"ip_address_from_cmd"`
	IpAddressOnUpdateCmd string                 `toml:"ip_address_on_update_cmd"`
	ApiParams            map[string]interface{} `toml:"api_param"`
}

// AppConfig 包含所有配置
type AppConfig struct {
	Global  GlobalConfig   `toml:"global"`
	Records []RecordConfig `toml:"record"`
}

// ReadConfig 读取并解析配置文件
func ReadConfig(filePath string) (*AppConfig, error) {
	fmt.Printf("ConfigFilePath: %s\n", filePath)
	configData, err := ioutil.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("Error reading config file %s: %w", filePath, err)
	}

	var config AppConfig
	if _, err := toml.Decode(string(configData), &config); err != nil {
		return nil, fmt.Errorf("Error decoding TOML config: %w", err)
	}

	// 设置默认检查间隔
	if config.Global.CheckInterval <= 0 {
		config.Global.CheckInterval = 60 // 默认为60秒
	}

	return &config, nil
}

// Helper function to get string from ApiParams
func getStringParam(params map[string]interface{}, key string, required bool) (string, error) {
	val, ok := params[key]
	if !ok {
		if required {
			return "", fmt.Errorf("missing required api_param: %s", key)
		}
		return "", nil
	}
	strVal, ok := val.(string)
	if !ok {
		return "", fmt.Errorf("api_param %s is not a string, got: %T", key, val)
	}
	return strVal, nil
}

// Helper function to get int64 from ApiParams (handles int64 directly or string conversion)
func getInt64Param(params map[string]interface{}, key string, required bool) (int64, error) {
	val, ok := params[key]
	if !ok {
		if required {
			return 0, fmt.Errorf("missing required api_param: %s", key)
		}
		return 0, nil // Or a default value like -1 if 0 is meaningful
	}

	switch v := val.(type) {
	case int64:
		return v, nil
	case string:
		i, err := strconv.ParseInt(v, 10, 64)
		if err != nil {
			return 0, fmt.Errorf("api_param %s (string value %s) could not be parsed to int64: %w", key, v, err)
		}
		return i, nil
	default:
		return 0, fmt.Errorf("api_param %s is not a number or string representation of a number, got: %T", key, val)
	}
}

// Helper function to get bool from ApiParams
func getBoolParam(params map[string]interface{}, key string, required bool) (bool, error) {
	val, ok := params[key]
	if !ok {
		if required {
			// For bool, it's tricky if missing means false or error. Let's assume error if required.
			return false, fmt.Errorf("missing required api_param: %s", key)
		}
		return false, nil // Default to false if not present and not required
	}
	boolVal, ok := val.(bool)
	if !ok {
		// Try parsing from string if it's like "true" or "false"
		if strVal, isStr := val.(string); isStr {
			b, err := strconv.ParseBool(strVal)
			if err != nil {
				return false, fmt.Errorf("api_param %s (string value %s) could not be parsed to bool: %w", key, strVal, err)
			}
			return b, nil
		}
		return false, fmt.Errorf("api_param %s is not a boolean, got: %T", key, val)
	}
	return boolVal, nil
}
