import { fireEvent, render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import App from "./App.svelte";
import type { ApplicationSnapshot } from "./lib/contracts";

const noDeviceSnapshot: ApplicationSnapshot = {
  revision: 1,
  observedStatusAt: null,
  candidates: [],
  selectedCandidateId: null,
  verifiedPuppis: null,
  mutationsQualified: false,
  hostSharing: null,
  radios: [],
  recoveryRequired: false,
  deviceRole: null,
  clientEvidence: [],
  savedClients: [],
  savedClientsAvailable: true,
  savedClientsFailure: null,
  usb: { level: "unavailable", summary: "No Puppis candidate detected", guidance: null },
  sharing: { level: "unavailable", summary: "Host sharing not inspected", guidance: null },
  protocol: { level: "unavailable", summary: "No verified Puppis", guidance: null },
  configuration: { level: "unavailable", summary: "Device configuration unavailable", guidance: null },
  clients: { level: "unavailable", summary: "Client evidence unavailable", guidance: null },
  activeOperation: null,
  lastFailure: null,
};

test("the overview explains every independent status when no device is present", () => {
  render(App, { snapshot: noDeviceSnapshot });

  expect(screen.getByRole("heading", { name: "Overview" })).toBeInTheDocument();
  expect(screen.getByText("No Puppis candidate detected")).toBeInTheDocument();
  expect(screen.getByText("Host sharing not inspected")).toBeInTheDocument();
  expect(screen.getByText("No verified Puppis")).toBeInTheDocument();
  expect(screen.getByText("Device configuration unavailable")).toBeInTheDocument();
  expect(screen.getByText("Client evidence unavailable")).toBeInTheDocument();
  expect(screen.getByRole("heading", { name: "First-launch checklist" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Open Diagnostics" })).toBeInTheDocument();
});

test("fixed hotkeys navigate to diagnostics and the exact About attribution", async () => {
  render(App, { snapshot: noDeviceSnapshot });
  await userEvent.keyboard("{Alt>}4{/Alt}");
  expect(screen.getByRole("heading", { name: "Diagnostics" })).toBeInTheDocument();
  await userEvent.keyboard("{Alt>}a{/Alt}");
  expect(screen.getByRole("heading", { name: "About" })).toBeInTheDocument();
  expect(screen.getByText("Created collaboratively by Keary Chang + Codex AI (OpenAI).")).toBeInTheDocument();
  expect(screen.getByText(/licensed under the MIT License/)).toBeInTheDocument();
});

test("a single candidate still requires explicit selection", async () => {
  const onSelectCandidate = vi.fn();
  render(App, { snapshot: { ...noDeviceSnapshot, candidates: [
    { id: "usb-a", interfaceName: "enx001", displayName: "USB network adapter (enx001)", linkSpeed: "super_speed" },
  ] }, onSelectCandidate });
  expect(screen.queryByRole("button", { name: "Verify P1411 identity" })).not.toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Inspect USB network adapter (enx001)" }));
  expect(onSelectCandidate).toHaveBeenCalledWith("usb-a");
});

test("client evidence never calls a host neighbor currently connected", () => {
  render(App, { snapshot: { ...noDeviceSnapshot, clientEvidence: [
    { alias: "client-1", displayName: "Headset", hardwareAddress: "aa:bb:cc:11:22:33", addresses: ["192.168.137.20"], kind: "connected", saved: true },
    { alias: "client-2", displayName: "Unlabeled client", hardwareAddress: "de:ad:be:ef:00:01", addresses: ["192.168.137.21"], kind: "recently_observed", saved: false },
  ] } });

  expect(screen.getByText("Connected client")).toBeInTheDocument();
  expect(screen.getByText("Recently observed client")).toBeInTheDocument();
  expect(screen.getByText("Host network evidence; current Wi-Fi connection is not confirmed.")).toBeInTheDocument();
});

test("an observed client shows its name MAC addresses and can be explicitly saved", async () => {
  const onSaveClientLabel = vi.fn().mockResolvedValue(undefined);
  render(App, { snapshot: { ...noDeviceSnapshot, clientEvidence: [
    { alias: "client-1", displayName: "Unlabeled client", hardwareAddress: "02:00:00:00:02:01", addresses: ["192.168.137.20", "192.168.137.21"], kind: "recently_observed", saved: false },
  ] }, onSaveClientLabel });

  expect(screen.getByText("02:00:00:00:02:01")).toBeInTheDocument();
  expect(screen.getByText("192.168.137.20 · 192.168.137.21")).toBeInTheDocument();
  await userEvent.type(screen.getByLabelText("Label 02:00:00:00:02:01"), "Living Room Headset");
  await userEvent.click(screen.getByRole("button", { name: "Save Living Room Headset" }));
  expect(onSaveClientLabel).toHaveBeenCalledWith("02:00:00:00:02:01", "Living Room Headset");
});

test("a saved observed client can be renamed or forgotten", async () => {
  const onRenameSavedClient = vi.fn().mockResolvedValue(undefined);
  const onForgetSavedClient = vi.fn().mockResolvedValue(undefined);
  render(App, { snapshot: { ...noDeviceSnapshot, clientEvidence: [
    { alias: "client-1", displayName: "Living Room Headset", hardwareAddress: "02:00:00:00:02:01", addresses: ["192.168.137.20"], kind: "recently_observed", saved: true },
  ], savedClients: [
    { label: "Living Room Headset", hardwareAddress: "02:00:00:00:02:01", lastObservedAt: 1_721_920_000, addresses: ["192.168.137.20"] },
  ] }, onRenameSavedClient, onForgetSavedClient });

  const rename = screen.getByLabelText("Rename Living Room Headset");
  await userEvent.clear(rename);
  await userEvent.type(rename, "Headset");
  await userEvent.click(screen.getByRole("button", { name: "Rename" }));
  expect(onRenameSavedClient).toHaveBeenCalledWith("02:00:00:00:02:01", "Headset");
  await userEvent.click(screen.getByRole("button", { name: "Forget Living Room Headset" }));
  expect(onForgetSavedClient).toHaveBeenCalledWith("02:00:00:00:02:01");
});

test("empty client evidence explains passive discovery and still lists saved clients", () => {
  render(App, { snapshot: { ...noDeviceSnapshot, savedClients: [
    { label: "Living Room Headset", hardwareAddress: "02:00:00:00:02:01", lastObservedAt: 1_721_920_000, addresses: ["192.168.137.20"] },
  ] } });

  expect(screen.getByText(/appears after it sends local network traffic/i)).toBeInTheDocument();
  expect(screen.getByRole("heading", { name: "Saved clients" })).toBeInTheDocument();
  expect(screen.getByText("Living Room Headset")).toBeInTheDocument();
});

test("the header reports observation freshness without exposing internal revisions", () => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(1_721_920_030_000));
  render(App, { snapshot: { ...noDeviceSnapshot, observedStatusAt: 1_721_920_000 } });

  expect(screen.getByText("Observed status · Updated 30s ago")).toBeInTheDocument();
  expect(screen.queryByText(/Live state/)).not.toBeInTheDocument();
  vi.useRealTimers();
});

test("client evidence refreshes every 15 seconds only while Overview is visible", async () => {
  vi.useFakeTimers();
  const onRefreshClientEvidence = vi.fn().mockResolvedValue(noDeviceSnapshot);
  render(App, { snapshot: { ...noDeviceSnapshot, selectedCandidateId: "usb-a" }, onRefreshClientEvidence });

  await vi.advanceTimersByTimeAsync(14_999);
  expect(onRefreshClientEvidence).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);
  expect(onRefreshClientEvidence).toHaveBeenCalledWith(true);
  await fireEvent.click(screen.getByRole("button", { name: "Network" }));
  await vi.advanceTimersByTimeAsync(15_000);
  expect(onRefreshClientEvidence).toHaveBeenCalledOnce();
  vi.useRealTimers();
});

test("role transition confirmation names both roles and client disruption", async () => {
  const onSetDeviceRole = vi.fn().mockResolvedValue(undefined);
  render(App, { snapshot: { ...noDeviceSnapshot, deviceRole: "prism_pulse", mutationsQualified: true }, onSetDeviceRole });
  await userEvent.click(screen.getByRole("button", { name: "Wi-Fi" }));
  await userEvent.click(screen.getByRole("button", { name: "Switch to Wi-Fi hotspot mode" }));

  expect(screen.getByRole("dialog")).toHaveTextContent("PrismPulse mode");
  expect(screen.getByRole("dialog")).toHaveTextContent("Wi-Fi hotspot mode");
  expect(screen.getByRole("dialog")).toHaveTextContent("Connected clients will disconnect");
  await userEvent.click(screen.getByRole("button", { name: "Confirm role transition" }));
  expect(onSetDeviceRole).toHaveBeenCalledWith("wifi_hotspot", true);
});

test("one per-radio Apply sends all changed fields, clears the password, and locks conflicting controls", async () => {
  let finish!: () => void;
  const pending = new Promise<void>((resolve) => finish = resolve);
  const onApplyRadio = vi.fn().mockReturnValue(pending);
  const snapshot: ApplicationSnapshot = {
    ...noDeviceSnapshot,
    radios: [
      { band: "five_ghz", ssid: "Five", channel: "36", country: "CA", enabled: true, bandwidth: "160", ssidMutation: true, passwordMutation: true, qualifiedChannels: ["0", "36"], passwordUnavailableReason: null },
      { band: "two_point_four_ghz", ssid: "Two", channel: "6", country: "CA", enabled: true, bandwidth: "40", ssidMutation: true, passwordMutation: false, qualifiedChannels: ["0", "6"], passwordUnavailableReason: "Password restoration has not completed reversible qualification." },
    ],
  };
  render(App, { snapshot, onApplyRadio });
  await userEvent.click(screen.getByRole("button", { name: "Wi-Fi" }));

  expect(screen.getByDisplayValue("Five")).toBeInTheDocument();
  expect(screen.getByText("Password restoration has not completed reversible qualification.")).toBeInTheDocument();
  const password = screen.getByLabelText("New 5 GHz password");
  await userEvent.clear(screen.getByLabelText("5 GHz network name"));
  await userEvent.type(screen.getByLabelText("5 GHz network name"), "FiveNew");
  await userEvent.type(password, "NewSecret9!");
  await userEvent.click(screen.getByRole("button", { name: "Apply 5 GHz settings" }));

  expect(onApplyRadio).toHaveBeenCalledWith("five_ghz", {
    ssid: "FiveNew",
    password: "NewSecret9!",
  });
  expect(password).toHaveValue("");
  expect(screen.getByRole("status")).toHaveTextContent("Applying 5 GHz settings");
  expect(screen.getByRole("button", { name: "Apply 2.4 GHz settings" })).toBeDisabled();
  finish();
});

test("an external sharing profile is visible but has no destructive controls", async () => {
  const onEnableManagedSharing = vi.fn().mockResolvedValue(noDeviceSnapshot);
  const snapshot: ApplicationSnapshot = {
    ...noDeviceSnapshot,
    sharing: { level: "healthy", summary: "Host sharing works through an external profile", guidance: "The manager will observe this profile but never modify it." },
    hostSharing: {
      upstreamAvailable: true, upstreamDescription: "Host effective default route", profileKind: "external",
      profileName: "Wired connection 2", active: true, ipv4Shared: true,
      downstreamAddress: "192.168.137.1/24", ipv6Disabled: false, autoconnect: false,
      subnetConflict: false, authorization: "prompt",
    },
  };
  render(App, { snapshot, onEnableManagedSharing });
  await userEvent.click(screen.getByRole("button", { name: "Network" }));

  expect(screen.getByText("External sharing profile")).toBeInTheDocument();
  expect(screen.getByText("Wired connection 2")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Remove managed sharing" })).not.toBeInTheDocument();
  expect(screen.getByText(/will replace the active external downstream configuration/)).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Enable managed sharing" }));
  expect(onEnableManagedSharing).not.toHaveBeenCalled();
  expect(screen.getByRole("dialog")).toHaveTextContent("external profile remains unchanged");
  await userEvent.click(screen.getByRole("button", { name: "Replace with managed sharing" }));
  expect(onEnableManagedSharing).toHaveBeenCalledOnce();
});

test("a selected candidate is described as unverified until protocol identity succeeds", async () => {
  const onVerify = vi.fn();
  const snapshot: ApplicationSnapshot = {
    ...noDeviceSnapshot,
    selectedCandidateId: "usb-a",
    candidates: [{ id: "usb-a", interfaceName: "enx001", displayName: "USB network adapter (enx001)", linkSpeed: "super_speed" }],
    usb: { level: "healthy", summary: "SuperSpeed USB connection", guidance: null },
    protocol: { level: "unknown", summary: "Candidate not yet verified", guidance: "Verify its P1411 identity before changing device settings." },
  };
  render(App, { snapshot, onVerify });

  await userEvent.click(screen.getByRole("button", { name: "Verify P1411 identity" }));

  expect(onVerify).toHaveBeenCalledOnce();
  expect(screen.getByText("USB identity alone does not prove this is a P1411.")).toBeInTheDocument();
});

test("the user can select an ambiguous candidate without it being called a Puppis", async () => {
  const onSelectCandidate = vi.fn();
  const snapshot: ApplicationSnapshot = {
    ...noDeviceSnapshot,
    revision: 2,
    candidates: [
      { id: "usb-a", interfaceName: "enx001", displayName: "USB network adapter (enx001)", linkSpeed: "super_speed" },
      { id: "usb-b", interfaceName: "enx002", displayName: "USB network adapter (enx002)", linkSpeed: "usb2" },
    ],
    usb: { level: "attention", summary: "Select a Puppis candidate", guidance: "Choose the USB network adapter connected to the Puppis." },
  };
  render(App, { snapshot, onSelectCandidate });

  await userEvent.click(screen.getByRole("button", { name: "Inspect USB network adapter (enx002)" }));

  expect(onSelectCandidate).toHaveBeenCalledWith("usb-b");
  expect(screen.getByText("Candidate identity has not been verified.")).toBeInTheDocument();
});

test("diagnostics are previewed before an explicit export", async () => {
  const onPreviewDiagnostics = vi.fn().mockResolvedValue({
    preview: "{\n  \"firmware\": \"qualified\"\n}",
    suggestedName: "puppis-s1-diagnostics.json",
  });
  const onExportDiagnostics = vi.fn();
  render(App, { snapshot: noDeviceSnapshot, onPreviewDiagnostics, onExportDiagnostics });

  await userEvent.click(screen.getByRole("button", { name: "Diagnostics" }));
  await userEvent.click(screen.getByRole("button", { name: "Generate private preview" }));

  expect(screen.getByText(/qualified/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Export reviewed diagnostics" })).toBeInTheDocument();
  expect(onExportDiagnostics).not.toHaveBeenCalled();
});

test("slow status polling runs while open without calling configuration getters", async () => {
  vi.useFakeTimers();
  const onRefreshStatus = vi.fn().mockResolvedValue(noDeviceSnapshot);
  const onReadRadios = vi.fn();
  render(App, { snapshot: noDeviceSnapshot, onRefreshStatus, onReadRadios });

  await vi.advanceTimersByTimeAsync(15_000);

  expect(onRefreshStatus).toHaveBeenCalledOnce();
  expect(onReadRadios).not.toHaveBeenCalled();
  vi.useRealTimers();
});
