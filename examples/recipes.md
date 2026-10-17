## Immagini disponibili:
- `Fedora-Cloud-Base-Generic-44-1.7.x86_64.qcow2`
- `focal-server-cloudimg-amd64.img`

```shell
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/$FROM.qcow2 \
  /var/lib/libvirt/images/$TO.qcow2 60G
```

qemu-img create -f qcow2 -F qcow2 -b /var/lib/libvirt/images/focal-server-cloudimg-amd64.img /var/lib/libvirt/images/saubuntu20.04.img 60G

## Setup
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

echo 'export PATH=$HOME/llvm-project/build/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

## SSH
```shell
./target/debug/testing start safedora44 \
  --user franc \
  --private-key secrets/vm_ed25519
```