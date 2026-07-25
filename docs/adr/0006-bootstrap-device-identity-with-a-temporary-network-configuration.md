# Bootstrap device identity with a temporary network configuration

Because the generic ASIX USB identity cannot prove that a candidate is a Puppis and protocol identity may be unreachable before assigning the host its Puppis-side address, the user-selected candidate receives a temporary NetworkManager configuration protected by a timed checkpoint. A confirmed P1411 may then receive the managed system-wide profile; failed or contradictory identity rolls the temporary configuration back, and no Puppis device mutation is permitted during bootstrap.
