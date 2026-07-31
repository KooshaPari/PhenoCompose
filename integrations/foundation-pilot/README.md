# Foundation pilot fixture

`intent.json` is the provider-neutral handoff shared by the three layers.

This file is a **non-runnable fixture**. Its `a...a` composition digest and
`b...b` artifact digest are deliberate placeholders and must never be recorded
as a successful pilot receipt. Generate both values from the rendered plan and
attested OCI artifact at run time.

1. Render a Docker plan in PhenoCompose and replace the fixture digest with the
   rendered SHA-256.
2. Submit the JSON body to BytePort's authenticated `POST /mesh/workloads`.
3. Read it back with authenticated `GET /mesh/workloads` and verify the same owner,
   digest, artifact reference, backend, and placement.
4. Convert the same composition name/digest/backend to NanoVMS `DeployComposition`.
5. Compare BytePort's persisted desired state with NanoVMS's sandbox correlation
   labels before reconciliation.

The fixture intentionally contains no cloud credentials, provider IDs, or runtime
handles. Those remain inside BytePort provider adapters and NanoVMS runtime adapters.
