# Creative Coding The Hard Way - Ash Allocator

A toy GPU memory allocator written from scratch with Rust and Ash.

## Usage

The best and most up-to-date examples for usage can be found in the tests/
directory.

## Why This and Not That

Because I want to.

This repository is not meant to be a production-ready GPU allocator. It's
exclusively meant to be a semi-useful toy that can be used when experimenting
with Vulkan. The goal is to favor simplicity and readability over performance.

The [Vulkan Memory Allocator](https://gpuopen.com/vulkan-memory-allocator/) has
[rust bindings](https://github.com/gwihlidal/vk-mem-rs) and is always the better
choice when compared to the contents of this repository. If you're here because
you need a GPU memory allocator, then leave immediately and go use VMA.
