<script lang="ts">
  import { onMount } from "svelte";
  import type { ApplicationSnapshot, DeviceRole, DiagnosticsBundle, OperationFailure, RadioBand, RadioSnapshot, RadioUpdate, StatusDimension } from "./lib/contracts";

  export let snapshot: ApplicationSnapshot;
  export let onSelectCandidate: (candidateId: string) => void | ApplicationSnapshot | Promise<void | ApplicationSnapshot> = () => {};
  export let onVerify: () => void | ApplicationSnapshot | Promise<void | ApplicationSnapshot> = () => {};
  export let onPreviewDiagnostics: () => Promise<DiagnosticsBundle> = async () => ({ preview: "", suggestedName: "puppis-s1-diagnostics.json" });
  export let onExportDiagnostics: (path: string, approvedPreview: string) => void | Promise<void> = () => {};
  export let onInspectNetwork: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onEnableManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onDisableManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onRemoveManagedSharing: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onReadRadios: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onApplyRadio: (band: RadioBand, update: RadioUpdate) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onSetDeviceRole: (role: DeviceRole, confirmed: boolean) => Promise<ApplicationSnapshot | void> = async () => {};
  export let onRefreshClientEvidence: (telemetryVisible: boolean) => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onRefreshStatus: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let onAcceptRecoveryBaseline: () => Promise<ApplicationSnapshot> = async () => snapshot;
  export let subscribeSnapshots: (handler: (next: ApplicationSnapshot) => void) => Promise<() => void> = async () => () => {};
  export let loadSnapshot: (() => Promise<ApplicationSnapshot>) | null = null;
  let activeSection: "overview" | "network" | "wifi" | "diagnostics" = "overview";
  let diagnostics: DiagnosticsBundle | null = null;
  let exportPath = "puppis-s1-diagnostics.json";
  let ssidDrafts: Record<string, string> = {};
  let channelDrafts: Record<string, string> = {};
  let fiveGhzPassword = "";
  let transitionTarget: DeviceRole | null = null;
  let confirmManagedReplacement = false;
  let failure: OperationFailure | null = null;
  let operationPending = false;
  let pendingLabel = "";
  $: for (const radio of snapshot.radios) {
    if (ssidDrafts[radio.band] === undefined) ssidDrafts[radio.band] = radio.ssid;
    if (channelDrafts[radio.band] === undefined) channelDrafts[radio.band] = radio.channel;
  }

  onMount(() => {
    let unsubscribe: (() => void) | null = null;
    let disposed = false;
    subscribeSnapshots((next) => snapshot = next).then((stop) => {
      if (disposed) stop(); else unsubscribe = stop;
    }).catch(() => {});
    if (loadSnapshot) loadSnapshot().then((next) => snapshot = next).catch(showFailure);
    const telemetryTimer = window.setInterval(() => {
      if (!operationPending && activeSection === "overview" && snapshot.selectedCandidateId) {
        onRefreshClientEvidence(true).then((next) => snapshot = next).catch(() => {});
      }
    }, 5000);
    const statusTimer = window.setInterval(() => {
      if (!operationPending) onRefreshStatus().then((next) => snapshot = next).catch(() => {});
    }, 15000);
    return () => { disposed = true; unsubscribe?.(); window.clearInterval(telemetryTimer); window.clearInterval(statusTimer); };
  });

  async function selectCandidate(candidateId: string) {
    try { const next = await onSelectCandidate(candidateId); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function verifyIdentity() {
    try { const next = await onVerify(); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function previewDiagnostics() {
    try { diagnostics = await onPreviewDiagnostics(); exportPath = diagnostics.suggestedName; failure = null; }
    catch (error) { showFailure(error); }
  }

  async function exportDiagnostics() {
    if (!diagnostics) return;
    try { await onExportDiagnostics(exportPath, diagnostics.preview); failure = null; }
    catch (error) { showFailure(error); }
  }

  async function updateFrom(operation: () => Promise<ApplicationSnapshot>) {
    try { snapshot = await operation(); failure = null; }
    catch (error) { showFailure(error); }
  }

  async function runMutation(label: string, operation: () => Promise<ApplicationSnapshot | void>) {
    operationPending = true;
    pendingLabel = label;
    try { const next = await operation(); if (next) snapshot = next; failure = null; }
    catch (error) { showFailure(error); }
    finally { operationPending = false; pendingLabel = ""; }
  }

  function showFailure(error: unknown) {
    if (typeof error === "object" && error !== null && "code" in error && "message" in error && "guidance" in error) {
      failure = error as OperationFailure;
    } else {
      failure = { code: "unexpected_failure", message: "The operation could not be completed safely.", guidance: "Refresh current state and review diagnostics before trying again.", diagnosticReference: null };
    }
  }

  async function applyBand(radio: RadioSnapshot) {
    const update: RadioUpdate = {};
    if (ssidDrafts[radio.band] !== radio.ssid) update.ssid = ssidDrafts[radio.band];
    if (channelDrafts[radio.band] !== radio.channel) update.channel = channelDrafts[radio.band];
    if (radio.band === "five_ghz" && fiveGhzPassword) update.password = fiveGhzPassword;
    fiveGhzPassword = "";
    try { await runMutation(`Applying ${bandName(radio.band)} settings`, () => onApplyRadio(radio.band, update)); }
    finally { fiveGhzPassword = ""; }
  }

  const dimensions: Array<{ key: keyof ApplicationSnapshot; label: string }> = [
    { key: "usb", label: "USB link" },
    { key: "sharing", label: "Host sharing" },
    { key: "protocol", label: "Puppis identity" },
    { key: "configuration", label: "Configuration health" },
    { key: "clients", label: "Client evidence" },
  ];

  function statusFor(key: keyof ApplicationSnapshot): StatusDimension {
    return snapshot[key] as StatusDimension;
  }

  function bandName(band: RadioBand): string {
    return band === "five_ghz" ? "5 GHz" : "2.4 GHz";
  }

  function roleName(role: DeviceRole | null): string {
    if (role === "prism_pulse") return "PrismPulse mode";
    if (role === "wifi_hotspot") return "Wi-Fi hotspot mode";
    if (role === "wifi_adapter") return "Wi-Fi adapter mode";
    return "Unknown role";
  }

  function otherSupportedRole(role: DeviceRole): DeviceRole {
    return role === "prism_pulse" ? "wifi_hotspot" : "prism_pulse";
  }

  async function requestManagedSharing() {
    if (snapshot.hostSharing?.profileKind === "external" && snapshot.hostSharing.active) {
      confirmManagedReplacement = true;
      return;
    }
    await runMutation("Enabling managed sharing", onEnableManagedSharing);
  }
</script>

<svelte:head>
  <title>Puppis S1 Manager for Linux</title>
</svelte:head>

<div class="shell">
  <header>
    <div>
      <p class="eyebrow">Unofficial Linux utility</p>
      <h1>Puppis S1 Manager</h1>
    </div>
    <span class="revision" aria-label={`State revision ${snapshot.revision}`}>Live state · {snapshot.revision}</span>
  </header>

  {#if failure}<div class="failure-banner" role="alert"><div><strong>{failure.message}</strong><p>{failure.guidance}</p></div><button type="button" aria-label="Dismiss operation failure" on:click={() => failure = null}>×</button></div>{/if}
  {#if operationPending}<div class="operation-status" role="status" aria-live="polite">{pendingLabel}. Verification and recovery will finish before other changes are allowed.</div>{/if}

  <nav aria-label="Application sections">
    <button type="button" aria-current={activeSection === "overview" ? "page" : undefined} on:click={() => activeSection = "overview"}>Overview</button>
    <button type="button" aria-current={activeSection === "network" ? "page" : undefined} on:click={() => activeSection = "network"}>Network</button>
    <button type="button" aria-current={activeSection === "wifi" ? "page" : undefined} on:click={() => activeSection = "wifi"}>Wi-Fi</button>
    <button type="button" aria-current={activeSection === "diagnostics" ? "page" : undefined} on:click={() => activeSection = "diagnostics"}>Diagnostics</button>
  </nav>

  {#if activeSection === "overview"}
  <main id="overview">
    <div class="section-heading">
      <div>
        <p class="eyebrow">Current connection</p>
        <h2>Overview</h2>
      </div>
      <p>The manager observes first. Nothing changes until you request it.</p>
    </div>

    <section class="status-grid" aria-label="Operational status">
      {#each dimensions as dimension}
        {@const status = statusFor(dimension.key)}
        <article class:attention={status.level === "attention"}>
          <div class="status-title">
            <span class={`dot ${status.level}`} aria-hidden="true"></span>
            <h3>{dimension.label}</h3>
          </div>
          <p>{status.summary}</p>
          {#if status.guidance}<small>{status.guidance}</small>{/if}
        </article>
      {/each}
    </section>

    {#if snapshot.clientEvidence.length > 0}
      <section class="client-panel" aria-labelledby="client-evidence-heading">
        <div class="client-panel-heading"><div><p class="eyebrow">Generic client devices</p><h3 id="client-evidence-heading">Client evidence</h3></div><button class="secondary" type="button" on:click={() => updateFrom(() => onRefreshClientEvidence(true))}>Refresh client telemetry</button></div>
        <div class="client-list">
          {#each snapshot.clientEvidence as client}
            <div><strong>{client.alias}</strong><span>{client.kind === "connected" ? "Connected client" : "Recently observed client"}</span>{#if client.kind === "recently_observed"}<small>Host network evidence; current Wi-Fi connection is not confirmed.</small>{:else}<small>Currently reported active by a validated Puppis capability.</small>{/if}</div>
          {/each}
        </div>
      </section>
    {/if}

    {#if snapshot.candidates.length > 1 && !snapshot.selectedCandidateId}
      <section class="candidate-panel" aria-labelledby="candidate-heading">
        <div>
          <p class="eyebrow">Selection required</p>
          <h3 id="candidate-heading">Which adapter is connected to your Puppis?</h3>
          <p>Candidate identity has not been verified.</p>
        </div>
        <div class="candidate-list">
          {#each snapshot.candidates as candidate}
            <button type="button" on:click={() => selectCandidate(candidate.id)} aria-label={`Inspect ${candidate.displayName}`}>
              <strong>{candidate.displayName}</strong>
              <span>{candidate.linkSpeed === "super_speed" ? "SuperSpeed" : candidate.linkSpeed === "usb2" ? "USB 2" : "Speed unknown"}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    {#if snapshot.selectedCandidateId && !snapshot.verifiedPuppis}
      <section class="candidate-panel" aria-label="P1411 identity verification">
        <div>
          <p class="eyebrow">Read-only verification</p>
          <h3>Verify this candidate</h3>
          <p>USB identity alone does not prove this is a P1411.</p>
        </div>
        <div class="action-column">
          <button class="primary" type="button" on:click={verifyIdentity}>Verify P1411 identity</button>
          <small>No device setting will be changed.</small>
        </div>
      </section>
    {/if}

    {#if snapshot.verifiedPuppis}
      <section class="device-facts" aria-label="Verified Puppis details">
        <span><small>Model</small><strong>{snapshot.verifiedPuppis.model}</strong></span>
        <span><small>Firmware</small><strong>{snapshot.verifiedPuppis.firmware}</strong></span>
        <span><small>Device changes</small><strong>{snapshot.mutationsQualified ? "Qualified" : "Read-only"}</strong></span>
      </section>
    {/if}
  </main>
  {:else if activeSection === "network"}
    <main>
      <div class="section-heading"><div><p class="eyebrow">Host networking</p><h2>Network</h2></div><p>NetworkManager remains in control of routing, DHCP, DNS, NAT, and firewall behavior.</p></div>
      <button class="secondary" type="button" on:click={() => updateFrom(onInspectNetwork)}>Inspect NetworkManager state</button>
      {#if snapshot.hostSharing}
        <section class="network-card" aria-label="Host sharing details">
          <div><small>Profile ownership</small><strong>{snapshot.hostSharing.profileKind === "external" ? "External sharing profile" : snapshot.hostSharing.profileKind === "managed" ? "Managed sharing profile" : "No sharing profile"}</strong></div>
          <div><small>Profile</small><strong>{snapshot.hostSharing.profileName ?? "Not configured"}</strong></div>
          <div><small>Upstream</small><strong>{snapshot.hostSharing.upstreamAvailable ? snapshot.hostSharing.upstreamDescription ?? "Available" : "Unavailable"}</strong></div>
          <div><small>Downstream</small><strong>{snapshot.hostSharing.downstreamAddress ?? "Not configured"}</strong></div>
        </section>
        {#if snapshot.hostSharing.subnetConflict}<p class="warning">Resolve the Puppis subnet conflict before enabling managed sharing.</p>{/if}
        {#if snapshot.hostSharing.profileKind !== "managed"}
          <div class="network-action"><p>{snapshot.hostSharing.profileKind === "external" && snapshot.hostSharing.active ? "Enabling managed sharing will replace the active external downstream configuration. The external profile will not be modified or deleted." : "Enabling creates a system-wide app-owned profile after verification."} A normal desktop authorization prompt may appear.</p><button class="primary" type="button" disabled={operationPending || snapshot.hostSharing.subnetConflict || !snapshot.hostSharing.upstreamAvailable} on:click={requestManagedSharing}>Enable managed sharing</button></div>
        {:else}
          <div class="button-row">
            <button class="secondary" type="button" disabled={operationPending} on:click={() => runMutation("Disabling managed sharing", onDisableManagedSharing)}>Disable managed sharing</button>
            <button class="danger" type="button" disabled={operationPending} on:click={() => runMutation("Removing managed sharing", onRemoveManagedSharing)}>Remove managed sharing</button>
          </div>
        {/if}
      {/if}
      {#if confirmManagedReplacement}
        <div class="dialog-backdrop">
          <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="sharing-confirm-title" tabindex="-1">
            <p class="eyebrow">Replace active downstream configuration</p><h3 id="sharing-confirm-title">Enable managed sharing instead?</h3>
            <p>The external profile remains unchanged, but the app-owned profile will become active on this Puppis interface.</p>
            <div class="button-row"><button class="secondary" type="button" on:click={() => confirmManagedReplacement = false}>Keep external sharing</button><button class="primary" type="button" on:click={async () => { confirmManagedReplacement = false; await runMutation("Enabling managed sharing", onEnableManagedSharing); }}>Replace with managed sharing</button></div>
          </div>
        </div>
      {/if}
    </main>
  {:else if activeSection === "wifi"}
    <main>
      <div class="section-heading"><div><p class="eyebrow">Device configuration</p><h2>Wi-Fi</h2></div><p>Settings are read on demand. Existing passwords never enter this view.</p></div>
      {#if snapshot.recoveryRequired}<div class="recovery" role="alert"><p>Recovery required. Device mutations are locked until the current state is reconciled.</p><button class="secondary" type="button" on:click={() => updateFrom(onAcceptRecoveryBaseline)}>Read and accept current configuration as baseline</button></div>{/if}
      <button class="secondary" type="button" on:click={() => updateFrom(onReadRadios)}>Read current radio settings</button>
      {#if snapshot.deviceRole}
        <section class="role-card" aria-label="Device role">
          <div><small>Current role</small><strong>{roleName(snapshot.deviceRole)}</strong>{#if snapshot.deviceRole === "wifi_adapter"}<p>Wi-Fi adapter mode is recognized for diagnostics but cannot be selected.</p>{/if}</div>
          {#if snapshot.deviceRole !== "wifi_adapter"}
            <button class="secondary" type="button" disabled={operationPending || snapshot.recoveryRequired || !snapshot.mutationsQualified} on:click={() => transitionTarget = otherSupportedRole(snapshot.deviceRole!)}>Switch to {roleName(otherSupportedRole(snapshot.deviceRole))}</button>
          {/if}
        </section>
      {/if}
      <div class="radio-grid">
        {#each snapshot.radios as radio}
          <section class="radio-card" aria-label={`${bandName(radio.band)} settings`}>
            <div class="radio-heading"><div><p class="eyebrow">{bandName(radio.band)}</p><h3>{radio.ssid}</h3></div><span>{radio.country} · {radio.bandwidth} MHz</span></div>
            <form on:submit|preventDefault={() => applyBand(radio)}>
              <label>{bandName(radio.band)} network name<input aria-label={`${bandName(radio.band)} network name`} bind:value={ssidDrafts[radio.band]} disabled={operationPending || !radio.ssidMutation || snapshot.recoveryRequired} /></label>
              <label>{bandName(radio.band)} channel<select aria-label={`${bandName(radio.band)} channel`} bind:value={channelDrafts[radio.band]} disabled={operationPending || radio.qualifiedChannels.length === 0 || snapshot.recoveryRequired}>{#each radio.qualifiedChannels as channel}<option value={channel}>{channel === "0" ? "Automatic" : channel}</option>{/each}</select></label>
              {#if radio.passwordMutation && radio.band === "five_ghz"}
                <label>New 5 GHz password<input aria-label="New 5 GHz password" type="password" autocomplete="new-password" bind:value={fiveGhzPassword} disabled={operationPending || snapshot.recoveryRequired} /></label>
              {:else if radio.passwordUnavailableReason}
                <p class="capability-note">{radio.passwordUnavailableReason}</p>
              {/if}
              <button class="primary" type="submit" disabled={operationPending || snapshot.recoveryRequired || (!radio.ssidMutation && radio.qualifiedChannels.length === 0)}>Apply {bandName(radio.band)} settings</button>
            </form>
          </section>
        {/each}
      </div>
      {#if snapshot.radios.length === 0}<p>Verify a Puppis, then read current radio settings.</p>{/if}
      {#if transitionTarget}
        <div class="dialog-backdrop">
          <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="role-confirm-title" tabindex="-1">
            <p class="eyebrow">Disruptive device change</p><h3 id="role-confirm-title">Switch from {roleName(snapshot.deviceRole)} to {roleName(transitionTarget)}?</h3>
            <p>Connected clients will disconnect during this transition. The new role is reported only after read-back verification.</p>
            <div class="button-row"><button class="secondary" type="button" on:click={() => transitionTarget = null}>Cancel</button><button class="primary" type="button" disabled={operationPending} on:click={async () => { const target = transitionTarget!; transitionTarget = null; await runMutation("Switching device role", () => onSetDeviceRole(target, true)); }}>Confirm role transition</button></div>
          </div>
        </div>
      {/if}
    </main>
  {:else}
    <main>
      <div class="section-heading">
        <div><p class="eyebrow">Private by construction</p><h2>Diagnostics</h2></div>
        <p>Review the complete redacted artifact before choosing where to save it.</p>
      </div>
      <button class="primary" type="button" on:click={previewDiagnostics}>Generate private preview</button>
      {#if diagnostics}
        <pre class="diagnostics-preview" aria-label="Diagnostics preview">{diagnostics.preview}</pre>
        <label class="export-field">Export destination<input bind:value={exportPath} /></label>
        <button class="primary" type="button" on:click={exportDiagnostics}>Export reviewed diagnostics</button>
      {/if}
    </main>
  {/if}
</div>
