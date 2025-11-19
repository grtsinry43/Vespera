<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as echarts from "echarts";
    import type { ECharts } from "echarts";

    let {
        title,
        data,
        unit = "%",
        color = "#3b82f6",
    } = $props<{
        title: string;
        data: { timestamp: number; value: number }[];
        unit?: string;
        color?: string;
    }>();

    let chartContainer: HTMLDivElement;
    let chart: ECharts | null = null;

    // 格式化时间
    function formatTime(timestamp: number): string {
        const date = new Date(timestamp * 1000);
        return date.toLocaleTimeString("zh-CN", {
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    // 初始化图表
    function initChart() {
        if (!chartContainer) return;

        chart = echarts.init(chartContainer);

        // 排序数据：按时间戳升序排列（旧数据在左，新数据在右）
        const sortedData = [...data].sort((a, b) => a.timestamp - b.timestamp);

        const option: echarts.EChartsOption = {
            grid: {
                left: 50,
                right: 20,
                top: 20,
                bottom: 30,
            },
            xAxis: {
                type: "category",
                boundaryGap: false,
                data: sortedData.map((d) => formatTime(d.timestamp)),
                axisLine: {
                    lineStyle: {
                        color: "#71717a",
                    },
                },
                axisLabel: {
                    color: "#a1a1aa",
                    fontSize: 11,
                },
            },
            yAxis: {
                type: "value",
                min: 0,
                max: 100,
                axisLine: {
                    show: false,
                },
                axisTick: {
                    show: false,
                },
                splitLine: {
                    lineStyle: {
                        color: "#27272a",
                        type: "dashed",
                    },
                },
                axisLabel: {
                    color: "#a1a1aa",
                    fontSize: 11,
                    formatter: `{value}${unit}`,
                },
            },
            series: [
                {
                    type: "line",
                    smooth: true,
                    symbol: "none",
                    lineStyle: {
                        color: color,
                        width: 2,
                    },
                    areaStyle: {
                        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                            {
                                offset: 0,
                                color: color + "40",
                            },
                            {
                                offset: 1,
                                color: color + "00",
                            },
                        ]),
                    },
                    data: sortedData.map((d) => d.value),
                },
            ],
            tooltip: {
                trigger: "axis",
                backgroundColor: "rgba(0, 0, 0, 0.8)",
                borderColor: "transparent",
                textStyle: {
                    color: "#fff",
                },
                formatter: (params: any) => {
                    const point = params[0];
                    return `${point.name}<br/>${title}: ${point.value.toFixed(1)}${unit}`;
                },
            },
        };

        chart.setOption(option);
    }

    // 更新图表数据
    function updateChart() {
        if (!chart) return;

        // 排序数据：按时间戳升序排列（旧数据在左，新数据在右）
        const sortedData = [...data].sort((a, b) => a.timestamp - b.timestamp);

        chart.setOption({
            xAxis: {
                data: sortedData.map((d) => formatTime(d.timestamp)),
            },
            series: [
                {
                    data: sortedData.map((d) => d.value),
                },
            ],
        });
    }

    // 响应式更新
    $effect(() => {
        // 当 data 变化时，重新初始化或更新图表
        if (chartContainer) {
            if (!chart) {
                // 如果图表还不存在，初始化
                initChart();
            } else if (data.length > 0) {
                // 如果图表存在且有数据，更新
                updateChart();
            } else {
                // 如果数据为空，清空图表
                chart.setOption({
                    xAxis: { data: [] },
                    series: [{ data: [] }],
                });
            }
        }
    });

    onMount(() => {
        // 响应窗口大小变化
        const resizeObserver = new ResizeObserver(() => {
            chart?.resize();
        });

        if (chartContainer) {
            resizeObserver.observe(chartContainer);
        }

        return () => {
            resizeObserver.disconnect();
        };
    });

    onDestroy(() => {
        chart?.dispose();
    });
</script>

<div bind:this={chartContainer} class="w-full h-[200px]"></div>
