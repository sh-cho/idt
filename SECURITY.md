
# Security Policy

## Reporting a security issue

Please report security issue privately using GitHub's private security reporting system (https://github.com/sh-cho/idt/security)

Do not use public issue tracker for security issue

Please give us up to 90 days to remediate the vulnerability before public disclosure.


## Supply chain security

### SLSA Level 3 Build Provenance

Since `0.1.15`, all release artifacts include [SLSA Level 3](https://slsa.dev/spec/v1.0/levels#build-l3) build provenance. This provides a tamper-resistant, verifiable record that each artifact was built from a specific source commit in an isolated environment, using the [slsa-framework/slsa-github-generator](https://github.com/slsa-framework/slsa-github-generator).

The provenance file (`.intoto.jsonl`) is attached to each GitHub Release.

### Verifying artifacts

Install [slsa-verifier](https://github.com/slsa-framework/slsa-verifier#installation), download the artifact and provenance file from the [Releases](https://github.com/sh-cho/idt/releases) page, then verify:

```bash
slsa-verifier verify-artifact idt_linux_x86_64.tar.gz \
  --provenance-path multiple.intoto.jsonl \
  --source-uri github.com/sh-cho/idt \
  --source-tag <version>
```

For more information on SLSA, please refer to the [SLSA documentation](https://slsa.dev/).

### GitHub Attestations

Release artifacts are also signed with [GitHub Attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) (Sigstore). You can verify these using the GitHub CLI:

```bash
gh attestation verify idt_linux_x86_64.tar.gz --repo sh-cho/idt
```

### Container Image Signatures

Container images published to GHCR and Docker Hub are signed with [cosign](https://github.com/sigstore/cosign) using keyless signing (Sigstore/Fulcio). You can verify image signatures:

```bash
cosign verify ghcr.io/sh-cho/idt:<version> \
  --certificate-identity-regexp='https://github.com/sh-cho/idt' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com'
```
