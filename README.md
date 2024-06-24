# birb-vision
 Birb Vision

- [`birb-vision`](./crates/birb-vision/): the core crate
- **interfaces**: some provided interfaces
  - [`birb-vision-fake`](./crates/birb-vision-icube/): fake cameras for testing
  - [`birb-vision-icube`](./crates/birb-vision-icube/): the interface for the iCube cameras
  - [`birb-vision-mvs`](./crates/birb-vision-mvs/): the interface for the MVS cameras
  - [`birb-vision-nokhwa`](./crates/birb-vision-nokhwa/): the interface for the `nokhwa` crate
- [`birb-vision-bundle`](./crates/birb-vision-bundle/): wraps all the interfaces into a single crate
- [`birb-vision-explorer`](./crates/birb-vision-explorer/): ...