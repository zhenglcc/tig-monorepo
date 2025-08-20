import { Routes } from '@angular/router';
import { HomeComponent } from './pages/home/home.component';
import { ConfigComponent } from './pages/config/config.component';
import { SlaveMonitorComponent } from './pages/slave-monitor/slave-monitor.component';

export const routes: Routes = [
  {
    path: 'home',
    component: HomeComponent,
  },
  {
    path: 'config',
    component: ConfigComponent,
  },
  {
    path: 'slaves',
    component: SlaveMonitorComponent,
  },
  { path: '**', redirectTo: 'home' },
];
