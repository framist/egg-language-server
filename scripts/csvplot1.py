# 这个临时的脚本用于绘制测试项目的时长

import pandas as pd
import matplotlib.pyplot as plt


# 设置中文字体
plt.rcParams['font.sans-serif'] = ['Noto Sans CJK JP']

# 从CSV文件中读取数据，指定列名为"tests"和"times"
df = pd.read_csv('./scripts/mydata.csv', names=['test', 'time'])

df.sort_values(by=['time'], inplace=True, ascending=False)

# 提取测试名称和执行时间数据
tests = df['test']
times1 = df['time']

print(tests)

# 创建画布和子图对象
fig, ax = plt.subplots()


# 绘制第一组数据的柱状图
ax.bar(tests, times1, alpha=0.5, label='Data 1', log=True, color='black')

# 显示 y 轴网格线
ax.yaxis.grid(True, linestyle='--', which='major', color='gray', alpha=0.25)

# 添加标签和标题
ax.set_xlabel('测试项目')
ax.set_ylabel('时长 (s)')

ax.set_xticklabels(tests, rotation=45, ha='right')


# tight layout
plt.tight_layout()

# 展示图形
plt.show()

# 保存图形
# fig.savefig('./scripts/compare.png')
