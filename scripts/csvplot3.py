import pandas as pd
import matplotlib.pyplot as plt
import os
import seaborn as sns

plt.rcParams['font.sans-serif'] = ['Noto Sans CJK JP']


# 读取数据
os.chdir(os.path.join(os.path.dirname(__file__)))
# df = pd.read_csv("exprslen-times.csv")
df = pd.read_csv("exprslen-times-r.csv")
# 去处坏值
# df = df[df['times'] < 2]

exprslen = df["exprslen"]
times = df["times"]
xlabel = "输入程序的长度"
ylabel = "给出优化建议花费的时间（秒）"

# 计算相关系数
print(df.corr())



fig, ax = plt.subplots()


# 绘制散点图
ax.scatter(exprslen, times, marker='.', alpha=0.6 , color="#bababa", s=40, label='times')
# 绘制直方图
# ax.hist(exprslen, bins=20, color="#bababa", label='times', alpha=0.6, rwidth=0.8)


# 根据 exprslen 将 df 数据分为 5 组
maxlen = df['exprslen'].max()
minlen = df['exprslen'].min()
dfs = [d for _, d in df.groupby(pd.cut(df['exprslen'], bins=5, labels=False))]



# 绘制小提琴图
ax.violinplot([dfs[i]['times'] for i in range(5)],
            #   showmeans=True, 
              showmedians=True,
              positions=[dfs[i]['exprslen'].mean() for i in range(5)], 
            #   showextrema=False,
              widths=20)
# # 设置透明度
# for pc in ax.collections:
# 	pc.set_alpha(0.9)
 

# ax.boxplot([dfs[i]['times'] for i in range(5)],
#            showmeans=True,
#            showfliers=False,
#            positions=[dfs[i]['exprslen'].mean() for i in range(5)],
#            widths=20)


# tight layout
plt.tight_layout()

# 展示图形
plt.show()

plt.figure(figsize=(5,5))
sns.violinplot([dfs[i]['times'] for i in range(5)],
              showmeans=True, 
              showmedians=True,
              positions=[dfs[i]['exprslen'].mean() for i in range(5)], 
              showextrema=True,
              inner='box',
              widths=20)
plt.xlabel(xlabel)
plt.xticks(ticks=[])
plt.ylabel(ylabel)
plt.title('exprslen-times')
plt.show()