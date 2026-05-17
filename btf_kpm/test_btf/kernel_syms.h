#ifndef __KERNEL_SYMS_H__
#define __KERNEL_SYMS_H__

#include <ksyms.h>
#include <linux/printk.h>
// BTF解析偏移宏
#define BTF_OFFSET_VAR(struct_name, member_name) \
    unsigned long struct_name##__##member_name##__offset \
        __attribute__((used, section(".data"))) = 0

struct kernel_symbols{
};

int init_kernel_symbols(struct kernel_symbols *sym);


#endif