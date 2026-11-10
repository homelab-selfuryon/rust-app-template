package deploy

import "github.com/homelab-selfuryon/platform-kit/app"

// What this app needs from the platform. The schema and the render are in
// platform-kit; see its README for every field and what it becomes.
app.#App & {
	// Equal to the repository name: it is the image name and the host name.
	name: "rust-app-template"
	// The owning team (GitHub team of the same name).
	team: "platform"
	port: 8080
	// small | medium | large: requests and limits.
	size: "small"
	// An HTTPRoute on the environment's Gateway.
	public: true
	// Non-secret configuration only; secrets are files (secrets: [...]).
	env: LOG_LEVEL: "info"

	// More, when the app needs it:
	//   database: postgres: {}                       CloudNativePG, credentials in /var/run/secrets/db/
	//   secrets: ["api-key"]                         OpenBao kv/<team>/<env>/<name>/api-key -> /var/run/secrets/app/api-key
	//   network: allowFrom: [{app: "other"}]     who may call this app (its token is verified)
	//   network: calls: [{app: "other"}]         apps this one calls (token, audience "other")
	//   network: kubeAPI: true                       needed to verify tokens of callers
	//   network: egressFQDN: [{host: "api.example.org"}]
}
