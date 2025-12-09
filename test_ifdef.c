#ifdef DEBUG
int debug_enabled = 1;
#else
int debug_enabled = 0;
#endif

#ifndef CONFIG_H
#define CONFIG_H
#endif

#if defined(WINDOWS)
int os_type = 1;
#elif defined(LINUX)
int os_type = 2;
#elif defined(MAC)
int os_type = 3;
#else
int os_type = 0;
#endif
