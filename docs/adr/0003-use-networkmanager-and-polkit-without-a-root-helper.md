# Use NetworkManager and polkit without a root helper

The unprivileged application will manage the Puppis-facing host network through NetworkManager's system D-Bus API and rely on NetworkManager/polkit for operation-specific authorization. V1 will not run the GUI as root, invoke `sudo` or `nmcli`, install a setuid component, or add an application-specific privileged daemon; a narrowly scoped helper may be reconsidered only if a required operation is proven impossible through NetworkManager.
