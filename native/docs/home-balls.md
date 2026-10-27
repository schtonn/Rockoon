# Homepage falling balls

The native homepage reads `3D Entities/Balls.nmo` and `Textures/Ball_{Wood,Stone,Paper}.bmp` from the selected installation. It uses the original mesh positions, normals and UVs, including the paper ball silhouette. No game artwork, game DLL, WebView, GPU backend or third-party runtime is added to the executable. If any required asset is missing, damaged or unsupported, the normal sky and launch controls remain usable without the decoration.

The bounded NMO reader supports the retail Virtools 2.1 container and version-7 state chunks needed for those three meshes only. File, inflation, object, vertex and index limits apply before allocating or using data. Format references are [LibCmo CKFileReader](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/CKFileReader.cpp), [CKStateChunkOthers](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/CKStateChunkOthers.cpp) and [CKMesh](https://github.com/yyc12345/libcmo21/blob/master/LibCmo/LibCmo/CK2/ObjImpls/CKMesh.cpp). The reference license is in [LibCmo-LICENSE.txt](LibCmo-LICENSE.txt). The temporary BMap tool used to independently inspect the original meshes is not a build or runtime dependency.

## Physics reference and scope

Parameters were read from the user's existing reverse-engineering outputs, without editing that project:

- `Desktop/ball/rl/reports/decoupling/transformation-initial-geometry.json`, `ball_profiles` (`Physicalize_GameBall`).
- `Desktop/ball/rl/reports/decoupling/level1.ivpbin`, the world header (gravity and PSI interval).
- `Desktop/ball/rl/docs/physics.md`, the native engine and replay limitations.

| Ball | Mass | Friction | Elasticity | Linear damping | Angular damping |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wood | 1.9 | 0.8 | 0.2 | 0.9 | 0.1 |
| Stone | 10 | 0.5 | 0.1 | 0.3 | 0.1 |
| Paper | 0.2 | 0.5 | 0.4 | 1.5 | 0.1 |

Gravity is `(0, -20, 0)`, sphere collision radius is 2, and the simulation advances at 1/66 s. The renderer targets 60 Hz and uses small 96-pixel CPU-rasterized sprites with interpolated source normals, bilinear repeat-addressed textures and a neutral directional light. A short, bounded pose prediction smooths frames between physics steps. The sky is pre-scaled into cached tiles so a moving ball only repaints nearby pixels. Sizes and emission timing are decoration choices.

This is a sphere-only decorative approximation, **not a port of the original IVP solver**. It uses analytic linear drag under constant gravity, solid-sphere inertia, sequential normal impulses, positional correction and Coulomb friction that generates angular motion. Contact elasticity and friction use geometric means. It omits the retail contact scheduler, sleeping, engine force controllers and full level geometry. Ball centers share one plane while each mesh rotates in 3D. No original-trajectory equivalence is claimed.

Balls fall through the bottom edge, rather than bouncing on an invisible floor or accumulating. There are at most eight live bodies. The original directory header stays clear and launch controls remain above the decoration. Animation stops when leaving Home, while unfocused/hidden, or while launching/running the owned game. Long gaps are capped, so resuming cannot trigger a burst of overdue simulation steps. Widget redraws do not publish per-frame application messages.

## Validation

Workspace tests cover momentum/energy in unequal-mass collisions, drag ordering and terminal speed, spin from glancing contacts, display-rate independence, long-run body bounds, and malformed container limits. Clippy and formatting checks apply to all targets.

An opt-in local asset check uses `ROCKOON_BALL_ASSET_TEST_ROOT` and validates these source counts: wood 316 vertices / 280 triangles; stone 390 / 720; paper 315 / 320. It also renders each model at multiple orientations. `ROCKOON_BALL_RENDER_DIR` optionally exports a raw RGBA frame for visual review. Normal tests do not require game files.

UI review and performance are checked using an isolated game fixture and a QA-owned launcher process. No real game is started, no registry protocol association is changed, and the desktop reverse-engineering runtime remains untouched. This iteration is a local test build only.
Local release review (1040 x 700, software renderer): approximately 16% of one CPU core during animation, 61 MiB RSS. Configuration idle and minimized-home samples were 0% of one core; sampled memory did not grow. These are local measurements, not a cross-machine performance guarantee. The release window was also reviewed at 840 x 580, after restoring its size, with START hover, the folder popup, and resume after minimizing.
