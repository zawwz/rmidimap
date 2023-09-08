# Format

yaml configuration format

```yaml
# Log all device connections
[ log_devices: <bool> | default = false ]

# Midi backend to use. Currently only alsa
[ driver: alsa ]

# Device definitions
devices:
    [ - <device_config> ... ]
```

### `<device_config>`

Definition of one device with its config and corresponding events.

```yaml
# Find device by name with literal string
[ name: <string> ]

# Find device by name with regex
[ regex: <regex> ]

# Find device by exact address
[ addr: <string> ]

# Max number of devices to connect for this device definition.
[ max_connections: <int> | default = inf ]

# Max length of event queue for processing.
[ queue_length: <int> | default = 256 ]

# Time interval between executions.
# Actual interval can be longer if execution is longer than this value.
# Supports time extensions, example: 1s, 100ms...
[ interval: <duration> | default = 0 ]

# Log all midi events of device
[ log_events: <bool> | default = false ]

# Commands to run on device connect
connect: 
    [ - <run_config> ... ]

# Commands to run on device disconnect
disconnect: 
    [ - <run_config> ... ]

# Definitions of executions on MIDI events
events:
    [ - <event_config> ... ]
```

### `<event_config>`

Definition of one MIDI event condition and its corresponding executions.
```yaml
# Max number of devices to connect for this device definition.
[ max_connections: <int> | default = inf ]

# Max length of event queue for processing.
[ queue_length: <int> | default = 256 ]

# Time interval between executions.
# Actual interval can be longer if execution is longer than this value.
# Supports time extensions, example: 1s, 100ms...
[ interval: <duration> | default = 0ms ]

# Commands to run on device connect
connect: 
    [ - <run_config> ... ]

# Commands to run on device disconnect
disconnect: 
    [ - <run_config> ... ]

# Definitions of executions on MIDI events
events:
    [ - <event_config> ... ]
```

### `<event_config>`

Definition of one MIDI event condition and its corresponding executions.
```yaml
# Max number of devices to connect for this device definition.
[ max_connections: <int> | default = inf ]

# Max length of event queue for processing.
[ queue_length: <int> | default = 256 ]

# Time interval between executions.
# Actual interval can be longer if execution is longer than this value.
# Supports time extensions, example: 1s, 100ms...
[ interval: <duration> | default = 0ms ]

# Commands to run on device connect
connect: 
    [ - <run_config> ... ]

# Commands to run on device disconnect
disconnect: 
    [ - <run_config> ... ]

# Definitions of executions on MIDI events
events:
    [ - <event_config> ... ]
```