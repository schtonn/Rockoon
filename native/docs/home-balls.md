# Homepage physics scene

The native home reads the selected game's original ball meshes and textures, plus the original checkpoint platform `A02_Floor_01_Mesh` from `3D Entities/Level/Level_01.nmo`. Its materials are `Floor_Side.bmp`, `Floor_Top_Checkpoint.bmp` and `Floor_Top_Profil.bmp`. The platform rotates 90 degrees and scales uniformly by 1.2; rendering and collision use the same transformed vertices. No game artwork, DLL, WebView, GPU backend or extra runtime dependency is bundled. Missing platform assets preserve the falling-ball scene; missing ball assets preserve the normal static home.

The bounded Virtools reader supports the retail version-8 containers and version-7 state chunks needed for these models. It reads positions, normals, UVs, triangles and material slots. File, inflation, object, vertex and index limits apply before use. References: [LibCmo CKFileReader](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/CKFileReader.cpp), [CKStateChunkOthers](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/CKStateChunkOthers.cpp), [CKMesh](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/ObjImpls/CKMesh.cpp); license in [LibCmo-LICENSE.txt](LibCmo-LICENSE.txt). The temporary BMap inspection tool is not a build/runtime dependency.

## Physics

Reference data came from the user's existing `Desktop/ball/rl/reports/decoupling` outputs, without modifying or starting that runtime:

- `transformation-initial-geometry.json`: `Physicalize_GameBall` profiles.
- `level1.ivpbin`: gravity, PSI interval and the `A02_Floor_01` collision material.
- `docs/physics.md`: original engine/replay limitations.

| Ball | Mass | Friction | Elasticity | Linear damping | Angular damping |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wood | 1.9 | 0.8 | 0.2 | 0.9 | 0.1 |
| Stone | 10 | 0.5 | 0.1 | 0.3 | 0.1 |
| Paper | 0.2 | 0.5 | 0.4 | 1.5 | 0.1 |

Gravity is `(0,-20,0)`, collision radius 2, fixed step 1/66 s. The platform's source friction is .7 and elasticity .3. This remains a decorative approximation, **not a port or trajectory-equivalent reproduction of IVP**: analytic linear drag, solid-sphere inertia, normal impulses, finite triangle closest-point contacts and Coulomb friction. Contact elasticity/friction use geometric means. Low-speed floor contacts suppress restitution to remain supported. Sphere friction operates in the full contact tangent plane and generates spin around all three axes.

The world is now three-dimensional. Birth positions span a radius-17–22 ring in X/Z above the window. Each shot targets a random point in the central 7×3 area of the platform over 1.1–1.4 seconds. Initial velocity is solved analytically with the ball-specific gravity and damping, preserving a clearly oblique approach without applying in-flight steering. Initial angular velocities span +/-4 rad/s on each axis. Balls bounce, roll, push each other and can fall off its actual curved edges; no invisible floor or walls are added. At most twelve bodies remain live, with off-screen bodies retired naturally. A 100-second original-platform check confirms continued emission instead of a stationary full-capacity scene.

A perspective camera projects the original meshes with interpolated normals, bilinear repeat-addressed textures and perspective-correct UVs. Static platform depth masks balls behind it, and contact shadows render on its top plane where supported. Bodies sort by camera depth. A bounded pose prediction smooths display frames between physics steps. Sky/platform tiles cache static pixels to avoid full-background software repainting.

Spawn height is solved in camera space above the actual window edge. Clipping uses the whole home viewport; the directory row does not cut a horizontal line through entering balls. The directory menu and START remain above the scene. Resizing changes the camera without stretching the physical platform or body positions.

## Checkpoint flames

`Textures/Particle_Flames.bmp` supplies the original particle shapes. The reverse-engineering export `gameplay-scripts.json`, `PC_TwoFlames_SmallFlameA/B script`, supplies the effect reference: 20 ms emission delay, 1000 +/-250 ms life, .008 +/-.003 units/ms upward speed, size 3 -> .1, initial RGBA (.9215687,.1647059,.2745098,1) fading to zero, SRCALPHA + ONE blending. Native emitters sit at the checkpoint's two fire mouths after the same model transform. Fifty particles per mouth keep the effect bounded. Cached fire sprites refresh at 30 Hz while ball rendering remains at 60 Hz. The mouths use different phase offsets.

The lightweight emitter preserves the source texture, color, shrinking, rising and fading behavior, with a deterministic pseudo-random distribution. It approximates the original particle engine. Since iced's image widget uses alpha-over, a local sprite encodes the equivalent additive result against the cached static sky/platform. Fire renders behind the balls; it does not interact physically. Missing fire texture preserves the platform without a placeholder drawing.

## Lifecycle and validation

Only home widget redraws drive animation, with no per-frame application messages. Leaving Home, losing focus, hiding/minimizing or launching/running the owned game pauses balls and flames. Long gaps do not trigger a backlog of simulation steps.

Tests cover unequal-mass collision momentum/energy, drag ordering, frame-rate-independent stepping, glancing spin, finite platform support/bounce/edge escape, depth-axis collisions, camera inversion, births above the top edge, ring bounds, visible lateral motion and unobstructed arrival on the platform. Optional source-asset tests use `ROCKOON_BALL_ASSET_TEST_ROOT`; source mesh counts are wood 316/280, stone 390/720, paper 315/320 vertices/triangles. An original-platform 100-second run checks finite states, actual support, depth movement, bounded bodies and more than 45 births. Normal tests do not require game files.

UI checks use an isolated fixture and a QA-owned launcher. No real game starts, registry protocol association changes or desktop reverse-engineering files are modified. Builds remain local for user testing; no upload/release is performed.
## Mouse spring

Left-press a visible ball to start a spring interaction. The picking order uses camera depth and the projected sphere radius. The grab keeps the initial screen offset so clicking away from the center does not teleport the ball. Each fixed physics step applies `F = 80 * displacement - 1.5 * sqrt(80 * mass) * planar_velocity` in the camera plane, then keeps the normal gravity, air drag and collisions. Distance is converted from pixels with the current perspective scale. Ball IDs keep the held ball stable when other bodies leave the scene. Release preserves velocity; focus loss, leaving the window, changing assets or resizing clears the grab.

Logic checks cover force proportionality, camera-plane direction, motion for all three masses and inertia on release. UI behavior is left for user testing as requested.
