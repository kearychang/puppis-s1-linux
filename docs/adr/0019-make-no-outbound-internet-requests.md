# Make no outbound internet requests

V1 communicates only with local system services and the Puppis endpoint: it includes no analytics, crash reporting, update checks, vendor API calls, or remote web content. Package updates remain external to the application and support links open only after explicit user action, trading automated update and support telemetry for a small, auditable network surface in an application that handles local credentials and privileged network requests.
