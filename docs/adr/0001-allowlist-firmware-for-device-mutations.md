# Allowlist firmware for device mutations

Device mutations are enabled only on specifically qualified P1411 firmware builds; unknown, older, and newer builds remain read-only until they pass fixture checks and reversible validation. A minimum-version rule would reach more devices, but the proprietary protocol has no established compatibility guarantee and mutation evidence currently covers only one firmware build, so v1 favors recoverability over optimistic compatibility.
