# 这个脚本用于对 egg 支持的源码优化作语法模糊测试

from fuzzingbook.Grammars import EXPR_EBNF_GRAMMAR, srange, convert_ebnf_grammar, Grammar
from fuzzingbook.GrammarCoverageFuzzer import GrammarCoverageFuzzer
import subprocess
import string
import os
import time
import tqdm

import pandas as pd
import matplotlib.pyplot as plt

EXPR_EBNF_GRAMMAR: Grammar = {
    "<start>":
        ["<expr>"],

    "<expr>":
        ["<term> + <expr>", "<term> - <expr>", "<term>"],

    "<term>":
        ["<factor> * <term>", "<factor> / <term>", "<factor>"],

    "<factor>":
        ["<factor>", "(<expr>)", "<integer>", "<simbol>"],

    # "<sign>":
    #     ["+", "-"],

    "<integer>":
        ["<digit>+"],
        
    "<simbol>":
        ["<letter>+"],

    "<digit>":
        srange(string.digits),
        
    "<letter>":
        srange(string.ascii_letters)
}

def plot():
    # 读取数据
    os.chdir(os.path.join(os.path.dirname(__file__)))
    df = pd.read_csv("exprslen-times.csv")
    exprslen = df["exprslen"]
    times = df["times"]
    xlabel = "exprslen"
    ylabel = "times"
    plt.rcParams['font.sans-serif'] = ['Noto Sans CJK JP']
    # 绘制散点图
    plt.scatter(exprslen, times, marker='o', alpha=0.7 , color='blue', s=40, label='times')
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.title('exprslen-times')
    plt.legend(loc='upper left')
    plt.show()
    
    # 小提琴图
    plt.violinplot(exprslen, showmeans=True, showmedians=True)
    plt.xlabel('exprslen')
    plt.ylabel('times')
    plt.title('exprslen-times')
    plt.legend(loc='upper left')
    plt.show()
    
    # 针对 exprslen 绘制直方图
    # plt.hist(exprslen, bins=20, color='steelblue', edgecolor='k', label='exprslen')
    # plt.xlabel('exprslen')
    # plt.ylabel('频数')
    # plt.title('exprslen-times')
    # plt.legend(loc='upper left')
    # plt.show()
    
    
    

def main():

    fexpr = GrammarCoverageFuzzer(convert_ebnf_grammar(EXPR_EBNF_GRAMMAR), 
                                  start_symbol="<start>", 
                                #   max_nonterminals=100
                                  )

    os.chdir(os.path.join(os.path.dirname(__file__)))

    # 编译 cargo build --release --example fuzz_helper
    assert subprocess.run(["cargo", "build", "--release", "--example", "fuzz_helper"]).returncode == 0

    exprs = []
    times = []

    FILE = os.path.join(os.path.dirname(__file__), "temp.py")
    REPEAT_TIMES = 10000
    for _ in tqdm.tqdm(range(REPEAT_TIMES)):
        expr = fexpr.fuzz()
        exprs.append(len(expr))
        # 写入文件
        with open(FILE, "w") as f:
            f.write(expr)

        # 执行文件
        arg = ["../target/release/examples/fuzz_helper", FILE]
        # 开始计时
        start = time.time()
        assert subprocess.run(arg, stdout= subprocess.DEVNULL).returncode == 0
        end = time.time()
        times.append(end - start)
        
    # 数据保存到 csv 文件
    df = pd.DataFrame({"exprslen": exprs, "times": times})
    df.to_csv("exprslen-times-10000.csv", index=False)
    
    


if __name__ == "__main__":
    main()
    # plot()