import { Component, inject, signal } from '@angular/core';
import { TigApisService } from '../../services/tig-apis.service';
import { TableModule } from 'primeng/table';
import { TagModule } from 'primeng/tag';

@Component({
  selector: 'app-slave-monitor',
  standalone: true,
  imports: [TableModule, TagModule],
  templateUrl: './slave-monitor.component.html',
  styleUrl: './slave-monitor.component.scss',
})
export class SlaveMonitorComponent {
  tigService = inject(TigApisService);
  slaves: any = signal([]);
  constructor() {
    this.tigService.slaveStats$.subscribe((data: any) => {
      this.slaves.set(data);
    });
    this.tigService.getSlaveStats();
  }
}
