# CKB-OS
Replacement of ZeroOs for Jolt.

## Description
ckb-os is a **bare-metal RISC-V64 runtime** that lets regular C/Rust programs (compiled
against musl libc) run inside a **zero-knowledge virtual machine** (zkVM) such as
SP1 or Jolt. Think of it as a tiny OS kernel — just enough to boot, allocate memory,
and handle syscalls — but purpose-built for the constraints of ZK proof generation.

A normal Linux program expects a kernel underneath it. In a zkVM there is no kernel.
ckb-os fills that gap with the absolute minimum implementation needed to keep musl
libc (and therefore any C/Rust code linked against it) happy.

