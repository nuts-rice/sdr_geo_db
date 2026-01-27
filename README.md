# SDR_DB

A native and web terminal user interface for recording and viewing radio contacts, and possibly sharing SDR with many other listeners. 


## Running

To get the api server running:
` cargo run --bin api --features=api`

To run the native application:
`cargo run --bin sdr_db`

To test your plugged in SDR for  receiving spectrum and calculating IQ data:
` cargo run --bin sdr_test "driver=$YOUR_SDR_DONGLE" $CENTER_FREQUENCY_IN_HERTZ`



## Serving

To serve the website (from the sdr_db_website directory):
`trunk serve`
And visit http://localhost:8080.

## Contributing

PRs welcome. 


