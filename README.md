# Go DDNS Keeper - Dynamic DNS Client

Go DDNS Keeper is a Dynamic Domain Name System (DDNS) client tool written in Go. It allows you to monitor IP address changes and automatically update your DNS records with various DNS service providers (such as Aliyun, Cloudflare) based on a configuration file.

## Key Features

* Supports multiple DNS providers (Currently implemented: Aliyun, Cloudflare).
* Fetches the current IP address using custom commands.
* Executes custom commands after a successful IP address update.
* Flexible configuration using TOML format.
* Independent configuration and management for multiple domain records.
* Background worker mode, launching separate monitoring goroutines for each record.

## Compilation

### Prerequisites

* Go installed (Version 1.18 or higher recommended).
* Your Go development environment configured (e.g., `GOPATH`, `GOROOT`).

### Compilation Steps

1.  **Get the Code**:
    Assume your project code is located in the `ddns-go` directory.

2.  **Download Dependencies**:
    Navigate to the project root directory (`ddns-go`) and execute the following command to download and organize the project's required libraries:
    ```
    go mod tidy
    ```

3.  **Compile the Program**:
    In the project root directory, execute the following command to compile:
    ```
    go build -o ddns-keeper main.go aliyun_dns.go cloudflare_dns.go config.go executor.go worker.go
    ```
    Alternatively, if your `main` package is well-organized, you can simply use:
    ```
    go build -o ddns-keeper .
    ```
    Upon successful compilation, an executable file named `ddns-keeper` (Linux/macOS) or `ddns-keeper.exe` (Windows) will be generated in the current directory.

## Running

### Basic Command

./ddns-keeper [arguments]


### Command-line Arguments

* `-config <file_path>` or `--config <file_path>`:
  Specifies the path to the configuration file.
  Default value: `/etc/ddk/config.toml` (Linux/macOS) or `config.toml` (if `defaultConfigPath` was modified in the code).

* `-c <file_path>`:
  A shorthand argument identical in function to `-config`.

**Argument Priority**: If both `-c` and `-config` are specified, the program will choose one based on its implementation logic (usually the one specified later or the more specific one).

### Environment Variable

* `DDK_CONFIG`:
  You can specify the configuration file path by setting this environment variable. If set, it will override the configuration file path specified via command-line arguments.

## Configuration

The program uses a TOML-formatted configuration file. By default, it attempts to load `/etc/ddk/config.toml`.

### 1. Global Configuration `[global]`

This section contains global settings for the program.

* `api_version = "V1"`
    * Description: API version identifier (currently mainly for user reference; the program itself may not strictly depend on this version number for logic switching).
    * Type: String
    * Example: `"V1"`

* `post_up_wait = 0`
    * Description: The number of seconds the program waits after startup before executing `post_up_cmd`. Can be used to wait for network connection stability.
    * Type: Integer (seconds)
    * Example: `5`

* `post_up_cmd = ""`
    * Description: Command(s) to execute after the program starts and waits for `post_up_wait` seconds. Can be a series of commands separated by semicolons (`;`).
    * Type: String
    * Example: `"echo DDNS service started > /var/log/ddns.log; systemctl restart my_service"`

* `check_interval_seconds = 60`
    * Description: The periodic interval in seconds for checking and updating the IP address for each DNS record.
    * Type: Integer (seconds)
    * Default: `60` (if not specified in the config file or if set to 0 or a negative number)
    * Example: `300` (checks every 5 minutes)

### 2. Record Configuration `[[record]]`

This section is used to define one or more DNS records that need dynamic updates. You can configure different DNS providers and parameters for each record. Multiple `[[record]]` blocks can be used.

* `domain_registrar = ""`
    * Description: Specifies the domain service provider for this record.
    * Type: String
    * Required: Yes
    * Possible values: `"aliyun"`, `"cloudflare"` (case-insensitive)
    * Example: `"aliyun"`

* `ip_address_from_cmd = ""`
    * Description: The command used to obtain the current public IP address. The program executes this command and uses its standard output as the IP address.
    * Type: String
    * Required: Yes
    * Example: `"curl -s ifconfig.me/ip"`, `"my_custom_ip_script.sh"`

* `ip_address_on_update_cmd = ""`
    * Description: Command(s) to execute after the IP address for this record is successfully updated. The placeholder `${IP_ADDRESS}` in the command will be replaced with the newly updated IP address. Can be a series of commands separated by semicolons (`;`).
    * Type: String
    * Required: No
    * Example: `"echo IP for home updated to ${IP_ADDRESS}; /usr/local/bin/notify_me.sh ${IP_ADDRESS}"`

* `api_param`
    * Description: An inline table containing parameters specific to the `domain_registrar`.
    * Type: Table (map)
    * Required: Yes

#### 2.1 Aliyun Parameters (`api_param` for `domain_registrar = "aliyun"`)

When `domain_registrar = "aliyun"`, the `api_param` table should contain the following fields:

* `key_id = ""`
    * Description: Aliyun AccessKey ID.
    * Type: String
    * Required: Yes

* `key_secret = ""`
    * Description: Aliyun AccessKey Secret.
    * Type: String
    * Required: Yes

* `domain_name = ""`
    * Description: Your main domain name. E.g., `example.com`. **This field is crucial.**
    * Type: String
    * Required: Yes

* `record_id = ""`
    * Description: The unique ID of the DNS record to be updated. You can obtain this from the Aliyun console.
    * Type: String
    * Required: Yes

* `record_rr = ""`
    * Description: The host record (subdomain prefix) of the DNS record. For example, for `home.example.com`, this value would be `"home"`. For the main domain itself (e.g., `example.com`), `@` is typically used.
    * Type: String
    * Required: Yes

* `record_type = ""`
    * Description: The type of the DNS record.
    * Type: String
    * Required: Yes
    * Possible values: `"A"` (IPv4), `"AAAA"` (IPv6)

* `record_ttl = "600"`
    * Description: The TTL (Time To Live) value for the DNS record, in seconds.
    * Type: String (the program will attempt to parse it as an integer) or Integer
    * Required: Yes
    * Example: `"600"`, `600`

* `record_line = "default"`
    * Description: The DNS resolution line. Defaults to `"default"`. Other possible values include `"telecom"`, `"unicom"`, `"mobile"`, `"overseas"`, etc. Refer to Aliyun documentation for specifics.
    * Type: String
    * Required: No (defaults to `"default"`)

* `endpoint = "alidns.aliyuncs.com"`
    * Description: The API endpoint for Aliyun. Usually does not need to be changed.
    * Type: String
    * Required: No (defaults to `"alidns.aliyuncs.com"`)

#### 2.2 Cloudflare Parameters (`api_param` for `domain_registrar = "cloudflare"`)

When `domain_registrar = "cloudflare"`, the `api_param` table should contain the following fields:

* `api_key = ""`
    * Description: Cloudflare Global API Key or an API Token with DNS editing permissions.
    * Type: String
    * Required: Yes

* `email = ""`
    * Description: Your Cloudflare account email address (if using Global API Key). This field might not be necessary if using an API Token, depending on the token's configuration (the current code implementation expects this field with the `api_key`).
    * Type: String
    * Required: Yes (for Global API Key authentication)

* `zone_id = ""`
    * Description: The Zone ID of your domain on Cloudflare.
    * Type: String
    * Required: Yes

* `record_type = ""`
    * Description: The type of the DNS record.
    * Type: String
    * Required: Yes
    * Possible values: `"A"`, `"AAAA"`

* `record_name = ""`
    * Description: The name of the DNS record (subdomain prefix). For example, for `sub.example.com`, this value is `"sub"`. For the root domain (`example.com`), typically use `@`.
    * Type: String
    * Required: Yes

* `domain = ""`
    * Description: Your main domain name. E.g., `example.com`.
    * Type: String
    * Required: Yes

* `record_ttl = 1`
    * Description: The TTL value for the DNS record. For Cloudflare, `1` means automatic TTL. Other values are specific seconds.
    * Type: Integer
    * Required: Yes
    * Example: `1` (automatic), `120` (2 minutes)

* `record_proxied = false`
    * Description: Whether to enable Cloudflare's proxy (CDN, orange cloud).
    * Type: Boolean (`true` or `false`)
    * Required: Yes

### 3. Example Configuration File (`config.toml`)

[global]
api_version = "V1-Go"
post_up_wait = 3 # Wait for 3 seconds
post_up_cmd = "echo DDNS Keeper (Go) Started"
check_interval_seconds = 300 # Check every 5 minutes

[[record]]
domain_registrar = "aliyun"
ip_address_from_cmd = "curl -s https://ipv4.icanhazip.com" # Get IPv4
ip_address_on_update_cmd = "echo Aliyun IPv4 updated to ${IP_ADDRESS} >> /var/log/ddns_updates.log"
[record.api_param]
key_id = "YOUR_ALIYUN_ACCESS_KEY_ID"
key_secret = "YOUR_ALIYUN_ACCESS_KEY_SECRET"
domain_name = "yourdomain.com"
record_id = "ALIYUN_RECORD_ID_FOR_IPV4"
record_rr = "home" # resolves to https://www.google.com/search?q=home.yourdomain.com
record_type = "A"
record_ttl = "600" # or 600
record_line = "default"

[[record]]
domain_registrar = "cloudflare"
ip_address_from_cmd = "curl -s https://api64.ipify.org" # Get IPv6

ip_address_on_update_cmd = "/opt/scripts/cloudflare_updated.sh ${IP_ADDRESS}"
[record.api_param]
api_key = "YOUR_CLOUDFLARE_API_KEY"
email = "your-cloudflare-email@example.com"
zone_id = "CLOUDFLARE_ZONE_ID"
record_name = "ipv6access" # resolves to ipv6access.anotherdomain.net
domain = "anotherdomain.net"
record_type = "AAAA"
record_ttl = 1 # Automatic TTL
record_proxied = false


## Logging

The program outputs operational logs (including IP address retrieval, DNS update attempts, error messages, etc.) to standard output. You can redirect this to a log file as needed.

For example, on Linux/macOS:

./ddns-keeper -c /path/to/config.toml > /var/log/ddns-keeper.log 2>&1 &


## License

This project is licensed under the MIT License. (Assuming MIT, please update if different)

---

Hopefully, this document helps you better use and understand the Go DDNS Keeper tool!