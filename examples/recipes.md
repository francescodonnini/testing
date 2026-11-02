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

# QUESTA CONFIGURAZIONE PRODUCE UNA BUILD TROPPO GRANDE
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

# ALTERNATIVA PIÙ LEGGERA
cmake -G Ninja \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DLLVM_ENABLE_PROJECTS="clang" \
    -DLLVM_ENABLE_LLD=ON \
    -DLLVM_TARGETS_TO_BUILD=X86 \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_ENABLE_ASSERTIONS=ON \
    -DLLVM_FORCE_ENABLE_STATS=ON \
    -DBUILD_SHARED_LIBS=ON \
    -DLLVM_INCLUDE_TESTS=OFF \
    -DLLVM_INCLUDE_EXAMPLES=OFF \
    -DLLVM_INCLUDE_BENCHMARKS=OFF \
    -DLLVM_PARALLEL_LINK_JOBS=2 \
    ../llvm


ninja all
```

## SSH
```shell
./target/debug/testing start safedora44 \
  --user franc \
  --private-key secrets/vm_ed25519
```

## TESTS
+ Montaggio del modulo:
  al modulo si deve fornire la regione degli indirizzi degli stack userspace, stack shadow e la grandezza dell'area che
  vale per entrambi.
```shell
insmod shdwstck_runtime.ko shdwstck_size=0x800000 \
  shdwstck_user_start=0x100000000000 shdwstck_area_size=0x8000000000 \
  shdwstck_start=0xffff800000000000
ls -l /dev/shdwstck
```
```shell
sudo insmod shdwstck_runtime.ko shdwstck_size=0x800000 shdwstck_user_start=0x100000000000 \
  shdwstck_area_size=0x8000000000 shdwstck_start=0xffff800000000000 debug_mask=$((1|2|4|16|256))
sudo dmesg -w &
```
+ Proviamo il pivot del main thread stack senza usare i pass LLVM
```shell
clang -O1 -fPIE -c prog.c -o prog.o
clang prog.o runtime/build/stckcpy.o -Wl,-e,relocate_stack \
      -Lruntime/build -Wl,-rpath,'$ORIGIN' -lruntime -o prog
./prog; echo $?; dmesg | tail
```

+ Pipeline completa
```shell
clang -O1 -fPIE -fpass-plugin=shadow_stack/build/ShadowStack.so \
      -S -emit-llvm prog.c -o prog.ll

llc -O1 -relocation-model=pic -stop-after=prologepilog prog.ll -o prog.mir
llc -load shadow_stack_x86/build/libX86ShadowStack.so \
    -run-pass=x86-shadow-stack prog.mir -o prog.ss.mir
llc -O1 -relocation-model=pic -start-after=prologepilog \
    prog.ss.mir -filetype=obj -o prog.o

clang prog.o runtime/build/stckcpy.o -Wl,-e,relocate_stack \
      -Lruntime/build -Wl,-rpath,'$ORIGIN' -lruntime -o prog
```

## POSTGRES
```shell
sudo dnf install bison flex perl
git clone --depth 1 --branch REL_17_6 https://github.com/postgres/postgres ~/postgres
```

### Alcuni risultati con Postgres 
```shell
[franc@localhost postgres]$ B=$HOME/pg-s4/bin
$B/initdb -D $HOME/pgdata-s4 -U postgres
$B/pg_ctl -D $HOME/pgdata-s4 -l $HOME/pg-s4.log -o "-p 5433" start
The files belonging to this database system will be owned by user "franc".
This user must also own the server process.

The database cluster will be initialized with locale "en_US.UTF-8".
The default database encoding has accordingly been set to "UTF8".
The default text search configuration will be set to "english".

Data page checksums are disabled.

creating directory /home/franc/pgdata-s4 ... ok
creating subdirectories ... ok
selecting dynamic shared memory implementation ... posix
selecting default "max_connections" ... 100
selecting default "shared_buffers" ... 128MB
selecting default time zone ... UTC
creating configuration files ... ok
running bootstrap script ... ok
performing post-bootstrap initialization ... ok
syncing data to disk ... ok

initdb: warning: enabling "trust" authentication for local connections
initdb: hint: You can change this by editing pg_hba.conf or using the option -A, or --auth-local and --auth-host, the next time you run initdb.

Success. You can now start the database server using:

    /home/franc/pg-s4/bin/pg_ctl -D /home/franc/pgdata-s4 -l logfile start

waiting for server to start.... done
server started
[franc@localhost postgres]$ $B/pgbench -p 5433 -U postgres -i -s 50 postgres
dropping old tables...
NOTICE:  table "pgbench_accounts" does not exist, skipping
NOTICE:  table "pgbench_branches" does not exist, skipping
NOTICE:  table "pgbench_history" does not exist, skipping
NOTICE:  table "pgbench_tellers" does not exist, skipping
creating tables...
generating data (client-side)...
vacuuming...                                                                                
creating primary keys...
done in 6.69 s (drop tables 0.00 s, create tables 0.01 s, client-side generate 4.43 s, vacuum 0.22 s, primary keys 2.02 s).
[franc@localhost postgres]$ $B/pgbench -p 5433 -U postgres -c 8 -j 4 -T 60 postgres
pgbench (17.6)
starting vacuum...end.
transaction type: <builtin: TPC-B (sort of)>
scaling factor: 50
query mode: simple
number of clients: 8
number of threads: 4
maximum number of tries: 1
duration: 60 s
number of transactions actually processed: 45925
number of failed transactions: 0 (0.000%)
latency average = 10.450 ms
initial connection time = 29.722 ms
tps = 765.531356 (without initial connection time)
[franc@localhost postgres]$ $B/pgbench -p 5433 -U postgres -c 8 -j 4 -T 60 postgres
pgbench (17.6)
starting vacuum...end.
transaction type: <builtin: TPC-B (sort of)>
scaling factor: 50
query mode: simple
number of clients: 8
number of threads: 4
maximum number of tries: 1
duration: 60 s
number of transactions actually processed: 56328
number of failed transactions: 0 (0.000%)
latency average = 8.523 ms
initial connection time = 6.486 ms
tps = 938.687279 (without initial connection time)
```
