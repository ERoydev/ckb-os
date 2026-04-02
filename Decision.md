# Decisions

1. Musl `__libc_start_main` requires `_init` and `_fini` to exist as callabe function pointers - (a3, a4). If they miss the linker is going to fail with an undefined symbol error:
- Tipicaly in normal linux program they come from `crti.o` and `crtn.o` object files that compiler links automatically. They run code from .init and .fini ELF sections.
- I compile with `-nostartfile` flag via `zeroos-build` and `crti.o/crtn.o` aren't linked. 
- They are no-ops because there is nothing for them to do, we have no global constructors or destructors, and in a zkVM guest there is no cleanup worth spending proof cycles on at exit.  

