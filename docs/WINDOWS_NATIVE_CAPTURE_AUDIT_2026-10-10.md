# Windows native capture audit

The Windows selector handles physical virtual-screen coordinates and uses GDI for capture. Current pixel acquisition uses one BitBlt for the entire selected rectangle, whereas ADR 0004 requires enumerating physical monitors, capturing each intersection, and composing tiles at their virtual-screen offsets. The pure geometry helpers for tiling already exist but are not integrated into pixel acquisition. Until that work is implemented and physically tested, cross-monitor correctness is not proven.

The selector message loop currently consumes WM_QUIT by exiting only the inner message-drain loop; it should terminate the selection operation safely instead.

Hosted CI run 38084026168 passed at branch SHA 98356a64a2c1381e74d0a0c33e12aaa3dc54071b. This is compile/test evidence, not physical desktop acceptance.
