import { Routes } from '@angular/router';
import { HomeComponent } from './pages/home/home.component';
import { ConfigComponent } from './pages/config/config.component';
import { MainnetInfoComponent } from './pages/mainnet-info/mainnet-info.component';

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
    path: 'network',
    component: MainnetInfoComponent,
  },
  { path: '**', redirectTo: 'home' },
];
