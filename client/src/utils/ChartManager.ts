import { ISeriesApi, createChart as createLightWeightChart, ColorType, UTCTimestamp, CrosshairMode, CandlestickSeries, HistogramSeries } from "lightweight-charts";

export class ChartManager {
    private priceSeries: ISeriesApi<"Candlestick">;
    private volumeSeries: ISeriesApi<"Histogram">;
    private lastUpdateTime: number = 0;
    private chart: any;
    private isDisposed: boolean = false;
    private currentBar: {
        close: number | null;
        volume: number | null;
    } = {
        close: null,
        volume: null,
    };

    constructor(
        ref: any,
        initialData: any[],
        layout: {background: string; color: string}
    ) {
        console.log('chart data: ', initialData);
        const chart = createLightWeightChart(ref, {
            autoSize: true,
            width: ref.clientWidth,
            height: ref.clientHeight,
            overlayPriceScales: {
              ticksVisible: true,
              borderVisible: false,
            },
            crosshair: {
              mode: CrosshairMode.Normal,
              vertLine: {
                width: 1,
                color: 'rgba(224, 224, 224, 0.5)',
                style: 2, // dashed
              },
              horzLine: {
                width: 1,
                color: 'rgba(224, 224, 224, 0.5)',
                style: 2, // dashed
              },
            },
            rightPriceScale: {
              visible: true,
              borderVisible: false,
              ticksVisible: true,
              entireTextOnly: false,
              scaleMargins: {
                top: 0.1,
                bottom: 0.1,
              },
              mode: 1, // Show all ticks
              autoScale: true,
              textColor: layout.color,
              fontSize: 12,
              minimumWidth: 60,
            },
            timeScale: {
              borderVisible: false,
              tickMarkFormatter: (time: UTCTimestamp) => {
                const date = new Date(time * 1000);
                return date.toLocaleDateString('en-US', {
                  month: 'short',
                  day: 'numeric',
                });
              },
            },
            grid: {
              horzLines: {
                color: 'rgba(197, 203, 206, 0.1)',
                visible: true,
              },
              vertLines: {
                color: 'rgba(197, 203, 206, 0.1)',
                visible: true,
              },
            },
            layout: {
              background: {
                type: ColorType.Solid,
                color: layout.background,
              },
              textColor: layout.color,
              fontSize: 12,
            },
          });
          this.chart = chart;

          // Price Series
          this.priceSeries = chart.addSeries(CandlestickSeries, {
            upColor: '#26a69a', // Green for bullish candles
            downColor: '#ef5350', // Red for bearish candles
            borderUpColor: '#26a69a',
            borderDownColor: '#ef5350',
            wickUpColor: '#26a69a',
            wickDownColor: '#ef5350',
          });
          this.priceSeries.priceScale().applyOptions({
            scaleMargins: {
              top: 0.1,
              bottom: 0.3,
            },
            borderVisible: false,
            ticksVisible: true,
            entireTextOnly: false,
            mode: 1, // Show all ticks
            autoScale: true,
            textColor: layout.color,
            fontSize: 12,
          });

          // Volume Series
          this.volumeSeries = chart.addSeries(HistogramSeries, {
            color: 'rgba(38, 166, 154, 0.5)', // Soft green with transparency
            priceFormat: {
              type: 'volume',
            },
            priceScaleId: '',
          });
          this.volumeSeries.priceScale().applyOptions({
            scaleMargins: {
              top: 0.9,
              bottom: 0,
            },
            borderVisible: false,
          });

          // Set initial data
          const priceData = initialData.map((data) => ({
            time: Math.floor(data.timestamp / 1000) as UTCTimestamp,
            open: data.open,
            high: data.high,
            low: data.low,
            close: data.close,
          }));

          const volumeData = initialData.map((data, index) => ({
            time: Math.floor(data.timestamp / 1000) as UTCTimestamp,
            value: data.volume,
            color:
              index > 0 && data.close >= initialData[index - 1].close
                ? 'rgba(38, 166, 154, 0.5)' // Green with transparency
                : 'rgba(239, 83, 80, 0.5)', // Red with transparency
          }));

          this.priceSeries.setData(priceData);
          this.volumeSeries.setData(volumeData);

          // Fit content and add margin
          chart.timeScale().fitContent();
    }

    public update(updatedPrice: any) {
        if (this.isDisposed || !this.chart) {
            console.warn('Attempted to update a disposed chart');
            return;
        }

        if (!this.lastUpdateTime) {
            this.lastUpdateTime = new Date().getTime();
        }

        // Update price series
        this.priceSeries.update({
            time: (this.lastUpdateTime / 1000) as UTCTimestamp,
            open: updatedPrice.open,
            high: updatedPrice.high,
            low: updatedPrice.low,
            close: updatedPrice.close,
        });

        // Update volume series
        this.volumeSeries.update({
            time: (this.lastUpdateTime / 1000) as UTCTimestamp,
            value: updatedPrice.volume,
            color:
                updatedPrice.close >= (this.currentBar.close || 0)
                    ? 'rgba(38, 166, 154, 0.5)' // Green with transparency
                    : 'rgba(239, 83, 80, 0.5)', // Red with transparency
        });

        // Update current bar
        this.currentBar = {
            close: updatedPrice.close,
            volume: updatedPrice.volume,
        };

        if (updatedPrice.newCandleInitiated) {
            this.lastUpdateTime = updatedPrice.time;
        }
    }

    public destroy() {
        if (this.isDisposed || !this.chart) {
            console.debug('Chart already disposed or null');
            return;
        }

        try {
            this.chart.remove();
            this.chart = null;
            this.isDisposed = true;
        } catch (error) {
            console.error('Error while destroying chart:', error);
        }
    }
}