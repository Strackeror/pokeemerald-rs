# Birch
- Let them watch the intro (?)
- The presentation will mostly be in engine
- Practical ? No
- Can you stop me ? Nope :)

# Intro
- Experimental projects I've been working on over the last couple of months
- Integrating rust into the game Pokémon Emerald

# Summary
- First we'll talk about the game and the console it runs on 
- Then we'll talk a bit about why I wanted to do this
- And then we'll go technical, about how to make c and rust interact. I'll talk about different useful concepts with the more complex concepts last

- As I said, the game:
# Pokémon Emerald
- I don't think I need to introduce pokémon
- Started in 1996 with Red/Green/Blue
- Today highest grossing franchise

- We're working with Pokémon Emerald Version
- See Slide

- Decomps use specialized compilers
- Talk abou rt romhacking community

- All this runs on this fun device:
# The Gameboy Advance
- See slide
- Do a comparison with another

- Another interesting thing with this console it how it handles graphics:
# Sprites, Palettes and Backgrounds
- One of the last consoles to handle graphics this way
- 32768 colors (5 bits per color, 15 bits RGB)
- Magic offsets (?)
- Show the mgba debug screens

- Now that we know what we're running on:
# Why
- Got annoyed by writing headers
- This is not and industrial project, main philosophy of 'if it works it works'

# Why2
- Because I can !
- Next we're going technical
- And we're going to use another window for code because resolution is a _bit_ too low for code


# Rust to C, C to Rust
- Explain staticlib and linking a bit
- thankfully pokeemerald has a modern linker script
- C ABI compatible, but needs to be explicit

# Bindgen
- There's over 100k lines of C code in pokeemerald, so let's use bindgen to automate things
- Not perfect, issues with bitfields, defines, integer types

# Panic handlers
- Talk a bit about the never type
- Like an exception handler, but it has to crash
- Having an allocator gives us access to things like Box, Vec, Hashmap

# Unsafe
- show scope based resource management
- Talk about unsafe rust's pitfall and the rustonomicon

# Charmap
- Const is very limited still
- No traits is very annoying
- Have to precompile length on more complex cases
- Could have been proc macros as well, but that's boring, and slow to compile

# Async
- Show common pattern in C
- Talk about interrupts and why you don't want one frame to take too long
- Explain the concept of a coroutine (hopefully not too much)
- Show the rust version and how it looks a lot better
- We don't use a waker, we just poll every frame

# PhantomData
- Explain how the C is going to store the pointer in the static array
- We need to make sure not to destroy the data before we destroy the sprite
- But we don't actually want to take space in memory

- Starting the conclusion

# Problems
- C codebase was very inconsistent about ints
- But also quite a bit of pointer casts

- Mutable globals are very bad in rust
- Technically I should have done wrappers, but we're single threaded and I didn't bother
- Games are notoriously bad for code

- Mistakes in C can snowball in reallly weird bugs in rust
- Buffer overflow into the stack took me 2 days to find

- Rust takes quite a bit of space (?)

# Conclusion
- Interesting project, but mostly experimental, and I'm not a game designer
- You have to really understands the C part to make the rust part work well
- Learned a lot about the C ABI, Rust and C's memory model
- Rust is a bit overkill for this


# Thank you
- PANIC !
