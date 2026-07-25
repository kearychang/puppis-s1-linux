# Separate hardware mutation tests from normal automation

Normal tests, CI, and packaging checks never mutate physical Puppis hardware. Hardware-changing tests require a distinct explicit command, a detected qualified device, a clean preflight snapshot, and operation-specific confirmation, while factory reset remains separately authorized; fixture, state-machine, fake-adapter, frontend, and redaction tests run safely by default.
