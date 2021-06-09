# Process struct
- process id 
- mutex or atomic bool is_running
  - saved registers if stopped
- dynamic memory
  - available virtual memory: binary tree
  - some memory mapping descriptors
- growing stack
- `stdin` and `stdout`
- syscalls