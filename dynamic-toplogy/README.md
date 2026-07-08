# Dynamic Topology Example

This example demonstrates how set up the simulation bench to allow selecting
predefined bench topologies from a remote client by building the bench with
the appropriate configuration.

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