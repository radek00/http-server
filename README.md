# Simple Http Server
Simple http server writen from scratch in Rust. Implemented api endpoints allow for navigating file system directories, uploading and downloading files.

## Features
- Multi-threading
- Dynamic routing
- File upload/download
- Static files serving
- TLS/SSL support
- Colorful terminal logs
- CORS configuration
- Basic HTTP Auth

## Usage
```
Simlpe HTTP Server with TLS/SSL support. Implemented api endpoints allow for navigating file system directories, uploading and downloading files.

Usage: http-server [OPTIONS]

Options:
Options:
  -p, --port <port>          Sets the port number [default: 7878]
  -t, --threads <threads>    Sets the number of threads [default: 12]
  -c, --cert <cert>          TLS/SSL certificate
      --certpass <certpass>  TLS/SSL certificate password [default: ]
  -s, --silent               Disable logging
      --cors                 Enable CORS with Access-Control-Allow-Origin header set to *
      --ip <ip>              Ip address to bind to [default: 0.0.0.0] [default: 0.0.0.0]
  -a, --auth <auth>          Enable HTTP Basic Auth. Pass username:password as argument
  -h, --help                 Print help
  -V, --version              Print version
```
## Using the cert option
To use the cert option you have to:
1. Generate the certificate with the following command: ```openssl req -x509 -newkey rsa:4096 -keyout myKey.pem -out cert.pem -days 365```.
2. Genenrate the pkcs12 file with the following command: ```openssl pkcs12 -export -out keyStore.p12 -inkey myKey.pem -in cert.pem```
3. Pass the cert and certpass arguments like this ```http-server -c ./keyStore.p12 --certpass yourPassword```. Certpass option can be left blank if no password was set.