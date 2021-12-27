# dmsr_collector (wip)

P1 DSMR data collector for Smart Electricity Meters in The Netherlands

Written in Rust, it reads from a serial port, converts the raw data to raw frames and then uses 
`nom` to parse the values into usable data frames. Then writes the data to a database.
