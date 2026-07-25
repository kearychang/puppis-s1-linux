# Hide the proprietary protocol behind task-oriented operations

One deep Puppis module exposes verified state and task-oriented device operations rather than command names, raw frames, endpoint details, or complete proprietary setter schemas. TCP lifecycle, framing, polling, single-flight scheduling, firmware qualification, preservation of hidden fields, verification, and recovery remain inside the module, while Tauri adapts its interface instead of mirroring vendor getters and setters.
