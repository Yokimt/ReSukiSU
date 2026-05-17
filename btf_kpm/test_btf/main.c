#include <compiler.h>
#include <kpmodule.h>
#include <linux/printk.h>
#include <common.h>
#include <kputils.h>
#include <linux/string.h>
#include <linux/sched.h>
#include <linux/mm_types.h>
#include <asm/current.h>
#include <asm/ptrace.h>
#include <asm/processor.h>
#include "main.h"


KPM_NAME("btf_test");
KPM_VERSION("3.0.0");
KPM_LICENSE("GPL v2");
KPM_AUTHOR("ChiDanTa");
KPM_DESCRIPTION("KernelPatch BTF Test");

static long btf_init(const char *args, const char *event, void *__user reserved)
{
    pr_info("btf_test: initialized\n");
    return 0;
}

static long btf_control0(const char *args, char *__user out_msg, int outlen)
{
    pr_info("btf_test: mm_struct__pgd__offset :%d\n",mm_struct__pgd__offset);
    pr_info("btf_test: task_struct__pid__offset :%d\n",task_struct__pid__offset);
    return 0;
}

static long btf_control1(void *a1, void *a2, void *a3)
{
    return 0;
}

static long btf_exit(void *__user reserved)
{
    pr_info("btf_test: exited\n");
    return 0;
}

KPM_INIT(btf_init);
KPM_CTL0(btf_control0);
KPM_CTL1(btf_control1);
KPM_EXIT(btf_exit);