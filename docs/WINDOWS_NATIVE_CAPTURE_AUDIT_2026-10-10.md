# Windows native capture audit

The Windows selector handles physical virtual-screen coordinates and uses GDI for capture. ADR 0004 requires enumerating physical monitors, capturing each intersection, and composing tiles at their virtual-screen offsets. Pure geometry helpers exist, but physical cross-monitor correctness remains unverified.

The selector message loop must handle WM_QUIT by terminating selection safely.

Hosted CI run 38084026168 passed at 98356a64a2c1381e74d0a0c33e12aaa3dc54071b. This is not physical desktop acceptance.

CI run 38088174192 at d2fcffab4daca52ec6a0d826a108e7d102c027bf reported Windows Clippy errors in monitors.rs: unresolved BOOL import and missing null import. Repair and requalify before declaring the Windows build green.
