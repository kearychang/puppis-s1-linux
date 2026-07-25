# Puppis S1 Linux Manager

This context describes a Linux desktop manager for configuring a supported Puppis and preparing its host network for PC-VR use.

## Language

**Supported environment**:
Ubuntu 26.04 running the manager with an original PrismXR Puppis S1 (P1411). Other Linux distributions and Puppis revisions are outside the first release's compatibility promise; client devices remain generic and unrestricted.
_Avoid_: Linux in general, any Puppis

**Unsupported environment**:
Any environment outside the supported environment. It may receive non-mutating diagnostics, but the product makes no compatibility promise for it.
_Avoid_: Incompatible environment

**Qualified firmware**:
A specific P1411 firmware build on which the product's supported mutations have been demonstrated. Version ordering alone does not qualify a firmware build.
_Avoid_: Compatible firmware, recent firmware

**Qualified channel**:
A radio channel whose use has been validated for the current radio, qualified firmware, and reported country. A numerically valid channel is not necessarily qualified.
_Avoid_: Valid channel, supported channel

**Puppis candidate**:
A host-visible USB network device whose observable characteristics are consistent with a Puppis but whose P1411 identity has not been confirmed through the device protocol.
_Avoid_: Puppis, detected Puppis

**Verified Puppis**:
A Puppis candidate that has identified itself as a P1411 through the device protocol during the current connection.
_Avoid_: Candidate, ASIX adapter

**Client device**:
Any phone, headset, or other station using a wireless network provided by the Puppis. Device type may be presented as an uncertain hint but never determines network access.
_Avoid_: Quest, headset, station

**Connected client**:
A client device that the Puppis currently reports as active through a validated device-protocol capability.
_Avoid_: Neighbor, detected client

**Recently observed client**:
A possible client device represented by recent host-network evidence that does not prove a current wireless connection.
_Avoid_: Connected client, active client

**Device mutation**:
A requested change to configuration held by the Puppis, including radio settings, operating mode, or factory state.
_Avoid_: Write, setter

**Supported mutation**:
A device mutation whose complete request, outcome, verification, and recovery behavior have been demonstrated on the supported environment. Merely discovering a device command does not make it supported.
_Avoid_: Available command, implemented command

**Settings transaction**:
A reversible device mutation bounded by a known original state, a requested state, outcome verification, and restoration when the outcome is not acceptable.
_Avoid_: Settings save, update

**Factory reset**:
A configuration-erasing device mutation followed by a separate recovery process. It is not a settings transaction and carries no automatic restoration promise.
_Avoid_: Reset, restore defaults

**Recovery required**:
A condition in which the Puppis state cannot be verified or restored after a device mutation. Further device mutations are suspended until the state is reconciled.
_Avoid_: Failed, error

**Diagnostics bundle**:
A user-inspectable support artifact describing the host, Puppis, and application state while excluding credentials and replacing persistent device identifiers with bundle-local aliases.
_Avoid_: Log archive, debug dump

## Device roles

**PrismPulse mode**:
The Puppis role dedicated to the PC-to-headset network used for PC-VR streaming.
_Avoid_: Mode 1

**Wi-Fi hotspot mode**:
The Puppis role providing a general-purpose wireless hotspot.
_Avoid_: Mode 2, hotspot

**Wi-Fi adapter mode**:
The Puppis role providing Wi-Fi connectivity to the host computer. It is distinct from sharing the host's existing upstream connection through the Puppis.
_Avoid_: Mode 3, adapter mode

**Host internet sharing**:
The host routes an existing upstream internet connection to clients reached through the Puppis-facing network.
_Avoid_: Bridge, Wi-Fi adapter mode

**Managed sharing profile**:
A host internet-sharing configuration created and owned by the product. The product may reconcile or remove only managed sharing profiles.
_Avoid_: Puppis profile, shared connection

**External sharing profile**:
A host internet-sharing configuration not created by the product. It may satisfy the desired network state but remains outside the product's control.
_Avoid_: Existing profile, user profile

**Puppis subnet conflict**:
The host already has a route other than the Puppis-facing network for `192.168.137.0/24`, making the required Puppis management subnet ambiguous.
_Avoid_: Network error, occupied subnet
