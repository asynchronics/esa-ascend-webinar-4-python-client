# Fault Injection Example

This example demonstrates how to set up a simulation bench with a fault manager
model to allow injecting faults, such as disabled connections, via the python
client.

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

The server terminal will print out the reply packets received by the OBC or
a timeout message.