import TableBtn from '@/common_components/layout/table/tableBtn';
import { getIntl } from '@/utils';
import { ProfileOutlined } from '@ant-design/icons';
import { Modal } from 'antd';
import { useState } from 'react';
import TaskTerminal from './terminal';

export interface TaskOutputProps {
  id: string;
}

const TaskOutput: React.FC<TaskOutputProps> = (props) => {
  const intl = getIntl();
  const [isModalOpen, setIsModalOpen] = useState(false);

  return (
    <>
      <TableBtn
        icon={ProfileOutlined}
        tip={intl.get('pages.task.output.tip')}
        onClick={() => setIsModalOpen(true)}
      />
      <Modal
        title={intl.get('pages.task.output.title')}
        open={isModalOpen}
        footer={null}
        onCancel={() => {
          setIsModalOpen(false);
        }}
        width={1000}
        destroyOnClose={true}
        maskClosable={false}
      >
        <TaskTerminal {...props} />
      </Modal>
    </>
  );
};

export default TaskOutput;
