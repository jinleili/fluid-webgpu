
# Fluid Simulation

![poiseuille](screenshot/Poiseuille_480p.gif)

## Performance per frame
render size: full screen on iPhone, 1600px x 1200px on macOS.

#### Poiseuille flow with 100 * 75 lattice

Device | cpu cost  | gpu frame time
--------- | -------------
MBP 2018 (15-inch, i7, AMD Radeon Pro 560X) |  17%  | 3.8ms
MBP 2018 (15-inch, i7, Intel(R) UHD Graphics 630) |  21%  | 2.3ms
iPhone 6 plus  | 80% | 13.5ms
iPhone Xs Max  | 54% | 6.3ms

## Run 
```sh
# on macOS
cargo run --example demo

# build for iOS
cargo build --target aarch64-apple-ios

# run on iOS device
cd iOS
pod install
# then use Xcode to open fluid_demo.xcworkspace
```

