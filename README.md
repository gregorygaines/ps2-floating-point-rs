# PS2 Floating-point

## Purpose

I am currently writing my own PS2 emulator, and I'm currently wrapping my head around the monster of floating-point operations so I thought why not document my journey.
This repo will be updated with some findings as I understand more while emulating. That's all there is to it.

## Why is PS2 floating-point so hard?

The PS2 does not follow the [IEEE 754 specifications](https://en.wikipedia.org/wiki/IEEE_754) for floating-point numbers, so it makes debugging a pain since I have to experiment
using a PS2 as my baseline to learn what works.

## Blog Posts

I am writing a series as I go, but I'm very busy at the moment, so I probably won't have part 2 for a while.

1. [Emulating PS2 Floating-Point Numbers: IEEE 754 Differences (Part 1)](https://www.gregorygaines.com/blog/emulating-ps2-floating-point-nums-ieee-754-diffs-part-1/)
