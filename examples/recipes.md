## Immagini disponibili:
- `Fedora-Cloud-Base-Generic-44-1.7.x86_64.qcow2`
- `focal-server-cloudimg-amd64.img`

```shell
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/$FROM.qcow2 \
  /var/lib/libvirt/images/$TO.qcow2 60G
```

## Setup
### Fedora
```shell
cloud-init status wait

sudo dnf install -y gcc clang clang++ make cmake rsync git lld ninja "kernel-devel-$(uname -r)"

git clone -b release/20.x https://github.com/llvm/llvm-project

git fetch --tags

git checkout llvmorg-20.1.8

mkdir build && cd build

env CC=`which clang` CXX=`which clang++` cmake -G "Ninja" \
    -DLLVM_ENABLE_PROJECTS="clang" \
    -DLLVM_ENABLE_LLD=ON \
    -DLLVM_TARGETS_TO_BUILD=X86 \
    -DLLVM_ENABLE_ASSERTIONS=ON \
    -DLLVM_FORCE_ENABLE_STATS=ON \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DCMAKE_INSTALL_PREFIX=/opt/llvm-20 \
    -DLLVM_INCLUDE_BENCHMARKS=OFF \
    -DLLVM_INCLUDE_EXAMPLES=OFF \
    -DLLVM_INCLUDE_TESTS=OFF \
    -DCLANG_INCLUDE_TESTS=OFF \
    ../llvm

ninja all
```

### Ubuntu
```shell
cloud-init status wait

sudo apt update

sudo apt install -y gcc clang make cmake rsync git lld ninja-build "linux-headers-$(uname -r)"

sudo apt install -y ca-certificates gpg wget
wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null \
  | gpg --dearmor - | sudo tee /usr/share/keyrings/kitware-archive-keyring.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/kitware-archive-keyring.gpg] https://apt.kitware.com/ubuntu/ focal main' \
  | sudo tee /etc/apt/sources.list.d/kitware.list >/dev/null
sudo apt update
sudo apt install -y cmake
cmake --version

git clone -b release/20.x https://github.com/llvm/llvm-project

git fetch --tags

git checkout llvmorg-20.1.8

mkdir build && cd build

env CC=`which clang` CXX=`which clang++` cmake -G "Ninja" \
    -DLLVM_ENABLE_PROJECTS="clang" \
    -DLLVM_ENABLE_LLD=ON \
    -DLLVM_TARGETS_TO_BUILD=X86 \
    -DLLVM_ENABLE_ASSERTIONS=ON \
    -DLLVM_FORCE_ENABLE_STATS=ON \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DCMAKE_INSTALL_PREFIX=/opt/llvm-20 \
    -DLLVM_INCLUDE_BENCHMARKS=OFF \
    -DLLVM_INCLUDE_EXAMPLES=OFF \
    -DLLVM_INCLUDE_TESTS=OFF \
    -DCLANG_INCLUDE_TESTS=OFF \
    ../llvm

ninja all
```

## SSH
```shell
./target/debug/testing start safedora44 \
  --user franc \
  --private-key secrets/vm_ed25519
```