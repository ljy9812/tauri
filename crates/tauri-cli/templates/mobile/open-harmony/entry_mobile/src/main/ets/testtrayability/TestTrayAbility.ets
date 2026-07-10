import { StatusBarViewExtensionAbility } from "@kit.StatusBarExtensionKit";
import { UIExtensionContentSession, Want } from "@kit.AbilityKit";
import { hilog } from '@kit.PerformanceAnalysisKit';

const DOMAIN = 0x0000;

// Module-level reference to the active session, accessible by the page
let activeSession: UIExtensionContentSession | null = null;

export function hideSession(): void {
  if (activeSession) {
    activeSession.terminateSelf();
    hilog.info(DOMAIN, 'TestTrayAbility', 'Session terminated via terminateSelf()');
  }
}

export default class TestTrayAbility extends StatusBarViewExtensionAbility {
  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    hilog.info(DOMAIN, 'TestTrayAbility', 'onSessionCreate called');
    activeSession = session;
    session.loadContent('pages/TestTrayPage');
  }

  onSessionDestroy(): void {
    hilog.info(DOMAIN, 'TestTrayAbility', 'onSessionDestroy called');
    activeSession = null;
  }
}
