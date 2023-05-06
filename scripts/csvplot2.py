import pandas as pd
import matplotlib.pyplot as plt


# 设置中文字体
plt.rcParams['font.sans-serif'] = ['Noto Sans CJK JP']

# 从CSV文件中读取数据，指定列名为"tests"和"times"
df = pd.read_csv('./scripts/mydata.csv', names=['test', 'time'])
df_egg = pd.read_csv('./scripts/eggdata.csv', names=['test', 'time'])

merged = pd.merge(df, df_egg, on='test', how='inner')

print(merged)

# 排序
merged.sort_values(by=['time_x'], inplace=True, ascending=False)

print(merged)

# 提取测试名称和执行时间数据
tests = merged['test']
times1 = merged['time_x']
times2 = merged['time_y']

print(tests)

# 创建画布和子图对象
fig, ax = plt.subplots()


# 绘制第一组数据的柱状图
ax.bar(tests, times1, alpha=0.5, label='Data 1', log=True, color='gray')

# 绘制第二组数据的柱状图
ax.bar(tests, times2, alpha=0.5, label='Data 2', log=True, color='black')


# 添加标签和标题
ax.set_xlabel('测试项目')
ax.set_ylabel('时长 (s)')
# ax.set_title('与 Egg 示例对比测试')

ax.legend(['本文', '基准'])
ax.set_xticklabels(tests, rotation=45, ha='right')

ax.yaxis.grid(True, linestyle='--', which='major', color='gray', alpha=0.25)


# tight layout
plt.tight_layout()

# 展示图形
plt.show()

# 保存图形
# fig.savefig('./scripts/compare.png')
