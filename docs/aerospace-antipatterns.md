# Aerospace Anti-Patterns

This note captures recurring expensive mistakes across legacy aerospace and newer commercial space programs.

The goal is not to score ideology points for "old space" or "new space." The useful distinction is where each style tends to fail, what those failures cost, and which patterns should be rejected early in product and program design.

## Framing

Working definition used here:

- `old aerospace`: large, slower-moving, heavily governed programs where requirements, interfaces, certification, and reporting layers tend to accumulate
- `new aerospace`: faster-moving, commercially driven programs that optimize for iteration, cadence, lower cost, and vertical integration

These are tendencies, not clean categories. Boeing Starliner, for example, sits inside a modern commercial program structure while also carrying many legacy-program characteristics.

## Core Thesis

The broad pattern from the sources is:

- old-space failures often come from over-bureaucratizing uncertainty
- new-space failures often come from under-governing uncertainty
- both fail expensively when software, ground systems, and operations are treated as secondary to the vehicle

## Old-Aerospace Anti-Patterns

### 1. Requirements That Outrun Resources

Source-backed finding:

- GAO repeatedly identified the need to match resources to requirements in space acquisitions and noted that programs that fail to do so drift into cost growth and schedule delay.

Why it gets expensive:

- budget, staffing, test scope, and integration plans are all built on a fiction
- downstream organizations optimize against an impossible contract
- eventual rebaseline happens after real cost has already been burned

Source:

- GAO, *Space Acquisitions: Some Programs Have Overcome Past Problems, but Challenges and Uncertainty Remain for the Future* (April 29, 2015)  
  https://www.gao.gov/products/gao-15-492t

### 2. Optimistic Schedules Used as Governance Theater

Source-backed finding:

- GAO reported that NASA human exploration programs continued to operate against overly optimistic schedules, and also called out cost reporting that obscured real growth once scope movement was accounted for.

Why it gets expensive:

- the program keeps making local decisions under false time pressure
- integration and test get compressed first
- leadership loses the ability to distinguish true recovery from reporting artifacts

Source:

- GAO, *NASA Human Space Exploration: Persistent Delays and Cost Growth Reinforce Concerns over Management of Programs* (June 19, 2019)  
  https://www.gao.gov/products/gao-19-377

### 3. Fragmented Ownership Across Space, Ground, and User Segments

Source-backed finding:

- GAO found repeated misalignment between satellites, ground systems, and user equipment. In some cases, space assets were launched while supporting ground capability lagged badly enough to limit actual usefulness.

Why it gets expensive:

- the visible milestone is achieved while the mission capability is not
- “delivered” hardware carries storage, workaround, or underutilization costs
- integration debt moves from development into operations

Source:

- GAO, *Space Acquisitions: DOD Continues to Face Challenges of Delayed Delivery of Critical Space Capabilities and Fragmented Leadership* (May 17, 2017)  
  https://www.gao.gov/products/gao-17-619t

### 4. Weak Configuration Control on Safety-Critical Changes

Source-backed finding:

- NASA’s lessons-learned material on the STS-108/109/110 SSME controller coefficient issue describes communication failures and deficiencies in flight software verification and validation that allowed the wrong correction to fly on three missions.

Why it gets expensive:

- minor changes inherit “small change” handling while carrying system-level consequences
- teams stop reasoning from first principles because the patch looks localized
- repeated flights normalize a latent defect

Source:

- NASA Safety and Mission Assurance, Lessons Learned / Significant Incidents, STS-110 SSME software coefficient incident  
  https://sma.nasa.gov/SignificantIncidents/lessons-learned.html

### 5. Treating Flight Software as a Support Function Instead of a Primary Cost Driver

Source-backed finding:

- NASA’s software engineering handbook notes that flight software is a major cost and schedule driver and that concurrent software development with the rest of the flight system has contributed to significant errors, including mission loss.

Why it gets expensive:

- cost models understate the hardest part of the system
- test campaigns are planned around hardware milestones instead of software maturity
- off-nominal behavior gets discovered during integrated operations instead of before

Source:

- NASA Software Engineering Handbook, SWE-151 cost estimate conditions / lessons learned  
  https://swehb.nasa.gov/display/SWEHBVD/SWE-151%2B-%2BCost%2BEstimate%2BConditions

## New-Aerospace Anti-Patterns

### 1. Mistaking Speed for Validation

Source-backed finding:

- NASA’s initial Starliner OFT investigation said ground intervention prevented loss of vehicle in both major software-defect cases and that the detectability of the issues should have allowed them to be found before flight.

Why it gets expensive:

- the organization learns the wrong lesson from early success
- integrated failure modes are discovered only when the whole stack is live
- the cost of “fast iteration” explodes once the environment is crewed, orbital, or public

Source:

- NASA Commercial Crew Program, *NASA Shares Initial Findings from Boeing Starliner Orbital Flight Test Investigation* (February 7, 2020)  
  https://blogs.nasa.gov/commercialcrew/2020/02/07/nasa-shares-initial-findings-from-boeing-starliner-orbital-flight-test-investigation/

### 2. Scaling Cadence Faster Than Assurance and Infrastructure

Source-backed finding:

- GAO’s 2025 launch-range report says increased commercial use of federal ranges is straining infrastructure and that cost recovery mechanisms have not kept pace.

Why it gets expensive:

- throughput appears cheap until the bottleneck moves to shared infrastructure
- hidden dependence on public infrastructure becomes a scaling tax
- operational tempo outruns the maintenance and governance needed to sustain it

Source:

- GAO, *National Security Space Launch: Increased Commercial Use of Ranges Underscores Need for Improved Cost Recovery* (June 30, 2025)  
  https://www.gao.gov/products/gao-25-107228

### 3. Normalizing Workarounds Around Known Defects

Source-backed finding:

- NASA software guidance warns that unrepaired critical defects plus operational workarounds increase the risk of mission delays, user error, and system failure.

Why it gets expensive:

- the product becomes operationally dependent on tribal knowledge
- defect closure gets replaced with procedural compensation
- every future change compounds hidden interface risk

Source:

- NASA Software Engineering Handbook resource note, *R037 - Unrepaired Defects For Flight Release*  
  https://swehb.nasa.gov/x/EYHcD

### 4. Assuming Automation Removes the Need for Systems Engineering

Inference from sources:

- Aerospace Corporation describes increased launch cadence driving more automation, digital tooling, and model-driven workflows, but frames that as a way to free engineers for anomalies and difficult integration problems, not as a replacement for mission assurance.

Why it gets expensive:

- teams automate recurring work but leave cross-boundary judgment under-specified
- the organization scales nominal flow while anomalies still depend on fragile heroics
- “digital thread” becomes presentation without closed-loop engineering discipline

Source:

- The Aerospace Corporation, *Keeping Pace with a Rapidly Evolving Launch Landscape* (July 17, 2025)  
  https://aerospace.org/article/keeping-pace-rapidly-evolving-launch-landscape

Note:

- the anti-pattern statement above is an inference from the article plus the failure cases elsewhere in this note; the article itself argues for stronger tooling and assurance, not weaker assurance

### 5. Relearning Legacy Integration Lessons Under Commercial Branding

Inference from sources:

- commercial structure does not eliminate interface-management, verification, or end-to-end rehearsal problems. Starliner’s history shows that modern program framing can still fail in familiar ways when design, code, and test quality systems break down.

Why it gets expensive:

- the organization assumes “commercial” means the old failure modes no longer apply
- interface debt gets renamed rather than removed
- credibility loss compounds cost growth because recovery now has to rebuild trust as well as hardware/software

Sources:

- NASA Commercial Crew Program, initial Starliner OFT investigation  
  https://blogs.nasa.gov/commercialcrew/2020/02/07/nasa-shares-initial-findings-from-boeing-starliner-orbital-flight-test-investigation/
- NASA Commercial Crew updates on continued Starliner testing and propulsion-system evaluation in 2024 and 2025  
  https://blogs.nasa.gov/commercialcrew/2024/06/28/nasa-boeing-discuss-ground-testing-starliner-timeline/  
  https://www.nasa.gov/blogs/commercialcrew/2025/03/27/nasa-boeing-prepare-for-starliner-testing/

## Shared Failure Modes

These patterns cut across both camps:

- software treated as subordinate to hardware
- integration treated as a phase instead of a design property
- cost and schedule status reported in a way that hides true remaining risk
- ground/operations/user systems treated as downstream rather than co-equal
- workaround culture replacing defect closure

## Practical Interpretation

If the question is “where do expensive mistakes come from,” the answer is usually one of these:

1. pretending uncertainty is gone when it is not
2. moving risk across organizational boundaries instead of deleting it
3. compressing verification before the system is actually understandable
4. optimizing the visible artifact while neglecting its operational dependencies

## Design-Algorithm Lens

Using Forge's `design-algorithm` sequence sharpens this note from postmortem pattern-spotting into a usable decision rule.

Apply the sequence in order:

### 1. Question Every Requirement

Ask:

- which requirement is actually mission-critical, and which is inherited ceremony
- whether a schedule, certification step, interface, or reporting artifact exists to reduce real risk or merely to signal control
- whether a claimed acceleration target is tied to mission value or to optics

Anti-patterns caught here:

- requirements/resource mismatch
- optimistic schedules used as governance theater
- commercial-speed narratives that substitute for evidence

### 2. Delete Any Part Or Process You Can

Delete:

- duplicate reporting layers that do not change decisions
- interface boundaries that exist for org-chart reasons rather than system reasons
- manual workaround processes that are compensating for defects that should be removed
- “temporary” operational procedures that have become structural

Anti-patterns caught here:

- fragmented ownership across space, ground, and user systems
- workaround culture replacing defect closure
- integration debt being pushed into operations

### 3. Simplify And Optimize What Remains

Simplify:

- connector and signal definitions into schema-checked artifacts
- safety-critical parameter changes into explicit configuration-controlled paths
- software and test planning so software is treated as primary architecture, not support glue
- end-to-end readiness into a small number of honest gates instead of many symbolic ones

Anti-patterns caught here:

- weak configuration control on safety-critical changes
- ad hoc interface management
- opaque “done” definitions that hide missing ground or operational readiness

### 4. Accelerate Cycle Time

Accelerate only after the contract is clean:

- shorten the loop for integrated rehearsal
- shorten the loop for anomaly reproduction
- shorten the loop for interface validation
- shorten the loop for cost/schedule truth reaching decision-makers

Anti-patterns caught here:

- mistaking speed for validation
- scaling cadence faster than assurance and infrastructure
- deferring integrated testing until the highest-consequence phase

### 5. Automate Last

Automate only when:

- the interface is stable enough to encode
- the automation reduces real operator burden without hiding anomalies
- the system still becomes more observable, not less

Good automation targets:

- schema validation
- connector-family builders
- timing/register rendering from constrained inputs
- repetitive nominal verification and artifact generation

Bad automation targets:

- hiding unresolved interface ambiguity
- normalizing procedural workarounds around known defects
- generating confidence signals that exceed actual system understanding

### Smallest Surviving Contract

The narrowest useful rule that survives this sequence is:

- validate interfaces early
- integrate truthfully
- automate only the stable and observable parts

That rule applies to rockets, ground systems, software releases, and also to repo-local engineering tooling like Forge.

## What To Do Instead

Reject these behaviors early:

- requirements that cannot be resourced with margin
- milestone schedules that depend on best-case integration
- “done” definitions that exclude ground, user, or operations readiness
- flight releases carrying known critical defects plus procedural workarounds
- connector/interface definitions that are not machine-checkable
- automation narratives that do not improve anomaly handling and system understanding

Prefer these patterns:

- explicit requirement-to-resource matching
- honest schedule ranges with visible integration/test margin
- configuration control for all safety-critical parameter changes
- software-first risk accounting where software is mission architecture, not support glue
- end-to-end test and rehearsal before first high-consequence operation
- schema-validated interfaces and deterministic build/render flows for engineering artifacts

## Why This Matters For Forge

The lesson for tools like Forge is straightforward:

- do not build workflow surfaces that hide integration risk
- prefer deterministic local artifacts over informal operator knowledge
- validate interfaces and schemas before generation
- treat the surrounding system, not just the visible output, as the product

That is the same reason the `schemdraw` skill in this repo is moving toward local examples, deterministic helpers, and schema validation for harness and interface drawings.

## ICD / Harness / Interface-Control Implications

These aerospace anti-patterns become very concrete once the artifact is an interface control drawing, cable drawing, or harness specification.

### 1. The Drawing Cannot Be The Source Of Truth By Itself

Bad pattern:

- a diagram is treated as authoritative even though the real pin map, signal inventory, connector family, or mating assumptions live in email, tribal knowledge, or spreadsheet fragments

Why it fails expensively:

- the visible document looks complete while the actual interface contract is not
- connector and signal ambiguities are discovered during integration, bring-up, or test
- updates become manual synchronization exercises across multiple artifacts

Better pattern:

- keep the drawing generated from a schema-checked source or helper layer
- make connector family, pin count, required signals, and required mappings machine-checkable

### 2. Cosmetic Consistency Is Not Interface Determinism

Bad pattern:

- every ICD page looks polished, but each one is authored ad hoc with hand-placed anchors, inconsistent signal names, and connector-specific assumptions embedded in drawing code

Why it fails expensively:

- repeated diagrams conceal repeated mistakes
- signal-name drift creates false mismatches during integration
- reviewing the document does not reliably review the contract

Better pattern:

- use normalized signal names
- define connector builders once
- validate endpoint families and mappings before rendering

### 3. “Flexible” Harness Specs Usually Mean Hidden Ambiguity

Bad pattern:

- connector pinouts, shield handling, drain wires, power rails, and NC pins are left intentionally vague so teams can “adapt in implementation”

Why it fails expensively:

- ambiguity moves into procurement, assembly, or field integration
- each downstream team resolves uncertainty differently
- the final system inherits undocumented local decisions that are hard to reverse

Better pattern:

- make ambiguity explicit and force it into named schema choices
- distinguish required signals, optional signals, ignored pins, and forbidden mappings
- fail generation when the contract is incomplete instead of drawing a plausible-looking figure

### 4. Ground And Test Interfaces Need The Same Discipline As Flight Interfaces

Bad pattern:

- production or flight harnesses get disciplined interface control, but EGSE, test breakouts, lab adapters, and debug headers are treated as temporary exceptions

Why it fails expensively:

- the test path becomes the least trustworthy part of the system
- debug-time adapters create undocumented polarity, pinout, or shielding differences
- flight-like verification gets polluted by ad hoc lab infrastructure

Better pattern:

- treat test and operations harnesses as first-class interfaces
- generate their drawings from the same schema and connector families
- make deviations from flight wiring explicit and reviewable

### 5. Interface Drawings Should Shorten Integration Loops, Not Lengthen Them

Bad pattern:

- updating an ICD or cable drawing requires manual redraws, bespoke notation, or a specialist who understands the local style but not necessarily the system contract

Why it fails expensively:

- changes are delayed
- review focuses on the picture rather than the interface
- the drawing becomes stale because regeneration is expensive

Better pattern:

- use deterministic local source files
- encode the contract in reusable helpers
- regenerate drawings quickly after every interface change
- keep the review surface close to the actual declared pins, signals, and mappings

### Smallest Useful Rule For Interface Artifacts

For ICDs, harnesses, and cable drawings, the smallest useful rule is:

- define the interface as data or validated helper calls first
- render the drawing second

If the picture is easy to update but the contract is hard to validate, the workflow is backwards.
