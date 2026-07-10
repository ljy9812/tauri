import { hilog } from '@kit.PerformanceAnalysisKit';
import { hideSession } from '../testtrayability/TestTrayAbility';

const DOMAIN = 0x0000;

@Entry
@Component
struct TestTrayPage {
  @State message: string = 'QuickOperation Panel';
  @State clickCount: number = 0;

  build() {
    Column({ space: 16 }) {
      Text(this.message)
        .fontSize(18)
        .fontWeight(FontWeight.Bold)
        .fontColor('#333333')

      Text(`Click count: ${this.clickCount}`)
        .fontSize(14)
        .fontColor('#666666')

      Button('Click Me')
        .fontSize(14)
        .onClick(() => {
          this.clickCount++;
          hilog.info(DOMAIN, 'TestTrayPage', 'Button clicked, count: %{public}d', this.clickCount);
        })

      Button('Close Panel')
        .fontSize(14)
        .backgroundColor('#ff6666')
        .onClick(() => {
          hilog.info(DOMAIN, 'TestTrayPage', 'Close button clicked');
          hideSession();
        })
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .backgroundColor('#f5f5f5')
    .padding(20)
  }
}
