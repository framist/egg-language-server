# 这个临时的脚本用于绘制模糊测试得到的数据

import pandas as pd
import matplotlib.pyplot as plt
import os
import seaborn as sns

plt.rcParams['font.sans-serif'] = ['Noto Sans CJK JP']
plt.rcParams['font.size'] = 16


# 读取数据
os.chdir(os.path.join(os.path.dirname(__file__)))
# df = pd.read_csv("exprslen-times.csv")
df = pd.read_csv("exprslen-times-r-10000.csv")
# 去处坏值
df = df[df['times'] < 3]
# df = df[df['exprslen'] < 100]

exprslen = df["exprslen"]
times = df["times"]
xlabel = "输入程序的长度"
ylabel = "给出优化建议花费的时间（秒）"

# 计算相关系数
print(df.corr())



# sns.regplot(x=exprslen, y=times, data=df, scatter_kws={'alpha': 0.01})
# plt.xlabel(xlabel)
# plt.ylabel(ylabel)
# plt.title('全流程优化时间-输入程序的长度 关系图')
# plt.show()


fig, ax = plt.subplots()
# 绘制散点图
ax.scatter(exprslen, times, marker='o', alpha=0.1 , color="#bbbbbb", s=40, label='times')
# 计算回归直线


# 根据 exprslen 将 df 数据分为 5 组
# maxlen = df['exprslen'].max()
# minlen = df['exprslen'].min()
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
plt.xlabel(xlabel)
plt.ylabel(ylabel)
plt.title('全流程优化时间-输入程序的长度 关系图')
plt.tight_layout()
# 展示图形
plt.show()

plt.figure(figsize=(5,5))
sns.violinplot([dfs[i]['times'] for i in range(5)],
              showmeans=True, 
              showmedians=True,
              # positions=[dfs[i]['exprslen'].mean() for i in range(5)], 
              showextrema=True,
              inner='box',
              widths=20)
plt.xlabel(xlabel)
plt.xticks(ticks=[])
plt.ylabel(ylabel)
plt.title('无有分块优化算法')
plt.show()

# 通过 split 参数可以将小提琴图分成两部分，分别显示数据的分布情况

df1 = pd.read_csv("exprslen-times-10000.csv")
df2 = pd.read_csv("exprslen-times-r-10000.csv")
# 去处坏值
df1 = df1[df1['times'] < 3]
df2 = df2[df2['times'] < 3]

df1s = [d['times'] for _, d in df1.groupby(pd.cut(df1['exprslen'], bins=5, labels=False))]
df2s = [d['times'] for _, d in df2.groupby(pd.cut(df2['exprslen'], bins=5, labels=False))]

# 合并数据, 增加一列'group'，为本来其所在的组的序号
df1s = pd.concat([pd.DataFrame({'times': df1s[i], 'group': [i] * len(df1s[i]), '使用分块': '否'}) for i in range(5)])
df2s = pd.concat([pd.DataFrame({'times': df2s[i], 'group': [i] * len(df2s[i]), '使用分块': '是'}) for i in range(5)])

# 合并数据
df = pd.concat([df1s, df2s], ignore_index=True)

sns.violinplot(x="group", y="times", hue="使用分块",
               data=df, palette="binary_r", split=True, inner='quartile', linewidth=1, scale='count')

# 显示 y 轴网格线
plt.grid(axis='y', linestyle='--', alpha=0.5)

plt.xlabel("输入程序的长度（平均）")
plt.xticks(ticks=[i for i in range(5)], labels=['{:.2f}'.format(dfs[i]['exprslen'].mean()) for i in range(5)])
plt.ylabel(ylabel)
# plt.title('无有分块优化算法')
plt.show()