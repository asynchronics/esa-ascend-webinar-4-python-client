# Configuration Example

This example demonstrates the use of local TOML configuration files to configure a simulation bench via the python client.

The bench is set up in such a way to accept raw TOML files and parse the received configuration server-side.

## Usage

Start the server with:

```bash
cargo run
```

In a separate terminal run:

```
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python run.py
```

The server terminal will print out the parsed configuration.