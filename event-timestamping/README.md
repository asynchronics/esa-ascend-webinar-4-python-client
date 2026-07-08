# Event Timestamping Example

This example demonstrates how to use mapped connections and a clock reader to
include a timestamp with the events sent to an event sink.

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