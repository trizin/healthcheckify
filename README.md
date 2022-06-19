# Service Health Checker

Sends GET request to the server's health check endpoints and looks for the correct response code or body.

## Usage

Create a `config.json` file and a `.env` within the same directory of `rhealthchecker`.

### Set up env vars
- `BIND_ADDR`
Address and port to bind the HTTP server to. Default: `127.0.0.1:8080`

- `THREAD_COUNT`
Number of threads to use to handle HTTP requests. Default: 5

### Set up config file

Config file is an array of service configurations.

#### Service configuration

- `id` : unique service id, this will be used to get service status.
- `url` : the url which the request will be sent to
- `strategy` : health check strategy, either `statuscode` or `stringcontains`. Default is `statuscode`.
- `strategy_string` : the string to look for in the response body, required if the strategy is set to `stringcontains`
- `timeout` : health check interval in seconds. Default is 10 seconds.


Example service configuration:
```json
{
  "id": "my_service",
  "url": "http://localhost:3000/check",
  "strategy": "statuscode",
  "timeout": 10
}
```

Example config file:
```json
[
  {
    "id": "my_service_1",
    "url": "http://localhost:3000/check",
    "strategy": "statuscode",
    "timeout": 10
  },
  {
    "id": "my_service_2",
    "url": "http://localhost:3001/check",
    "strategy": "stringcontains",
    "strategy_string": "success",
    "timeout": 10
  }
]
```

### Query health status

To query a service's status. Send a get request to:
`GET http://{BIND_ADDRESS}/{NODE_ID}`

If the service is down, the server will return response with code 500. Otherwise, the server will return a response with code 200.