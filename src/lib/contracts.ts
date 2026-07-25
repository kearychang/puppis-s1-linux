export type StatusLevel = "healthy" | "attention" | "unavailable" | "unknown";

export interface StatusDimension {
  level: StatusLevel;
  summary: string;
  guidance: string | null;
}

export interface ApplicationSnapshot {
  revision: number;
  candidates: PuppisCandidate[];
  selectedCandidateId: string | null;
  verifiedPuppis: DeviceIdentity | null;
  mutationsQualified: boolean;
  hostSharing: HostSharingObservation | null;
  radios: RadioSnapshot[];
  recoveryRequired: boolean;
  deviceRole: DeviceRole | null;
  clientEvidence: ClientEvidence[];
  usb: StatusDimension;
  sharing: StatusDimension;
  protocol: StatusDimension;
  configuration: StatusDimension;
  clients: StatusDimension;
  activeOperation: string | null;
  lastFailure: OperationFailure | null;
}

export interface ClientEvidence {
  alias: string;
  kind: "connected" | "recently_observed";
}

export type DeviceRole = "prism_pulse" | "wifi_hotspot" | "wifi_adapter";

export type RadioBand = "five_ghz" | "two_point_four_ghz";

export interface RadioUpdate {
  ssid?: string;
  channel?: string;
  password?: string;
}

export interface RadioSnapshot {
  band: RadioBand;
  ssid: string;
  channel: string;
  country: string;
  enabled: boolean;
  bandwidth: string;
  ssidMutation: boolean;
  passwordMutation: boolean;
  qualifiedChannels: string[];
  passwordUnavailableReason: string | null;
}

export interface HostSharingObservation {
  upstreamAvailable: boolean;
  upstreamDescription: string | null;
  profileKind: "none" | "managed" | "external";
  profileName: string | null;
  active: boolean;
  ipv4Shared: boolean;
  downstreamAddress: string | null;
  ipv6Disabled: boolean;
  autoconnect: boolean;
  subnetConflict: boolean;
  authorization: "allowed" | "prompt" | "denied" | "unknown";
}

export interface DeviceIdentity {
  model: string;
  firmware: string;
  roleCode: string;
}

export interface DiagnosticsBundle {
  preview: string;
  suggestedName: string;
}

export interface OperationFailure {
  code: string;
  message: string;
  guidance: string;
  diagnosticReference: string | null;
}

export interface PuppisCandidate {
  id: string;
  interfaceName: string;
  displayName: string;
  linkSpeed: "super_speed" | "usb2" | "unknown";
}
