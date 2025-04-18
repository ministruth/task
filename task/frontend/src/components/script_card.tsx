import confirm from '@/common_components/layout/modal';
import Table from '@/common_components/layout/table';
import {
  CreatedAtColumn,
  IDColumn,
  SearchColumn,
  UpdatedAtColumn,
} from '@/common_components/layout/table/column';
import styles from '@/common_components/layout/table/style.less';
import TableBtn from '@/common_components/layout/table/tableBtn';
import { API_PREFIX } from '@/config';
import {
  checkAPI,
  deleleAPI,
  getAPI,
  getIntl,
  paramSort,
  paramTime,
  postAPI,
  putAPI,
  StringIntl,
  UserPerm,
} from '@/utils';
import {
  CaretRightOutlined,
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
} from '@ant-design/icons';
import { ParamsType } from '@ant-design/pro-components';
import { ActionType, ProColumns } from '@ant-design/pro-table';
import { FormattedMessage } from '@umijs/max';
import { Button } from 'antd';
import type { SortOrder } from 'antd/es/table/interface';
import { useRef } from 'react';
import TaskEditor from './editor';

const request = async (
  params?: ParamsType,
  sort?: Record<string, SortOrder>,
) => {
  const msg = await getAPI(`${API_PREFIX}/scripts`, {
    created_sort: paramSort(sort?.created_at) || 'desc',
    created_start: paramTime(params?.createdStart),
    created_end: paramTime(params?.createdEnd, true),
    updated_sort: paramSort(sort?.updated_at) || 'desc',
    updated_start: paramTime(params?.updatedStart),
    updated_end: paramTime(params?.updatedEnd, true),
    page: params?.current,
    size: params?.pageSize,
  });
  return {
    data: msg.data.data,
    success: true,
    total: msg.data.total,
  };
};

const handleDelete = (
  intl: StringIntl,
  ref: React.MutableRefObject<ActionType | undefined>,
  id: string,
  name: string,
) => {
  confirm({
    title: intl.get('pages.script.delete.title', {
      name: name,
    }),
    content: intl.get('app.confirm'),
    onOk() {
      return new Promise((resolve, reject) => {
        deleleAPI(`${API_PREFIX}/scripts/${id}`, {}).then((rsp) => {
          if (rsp && rsp.code === 0) {
            ref.current?.reloadAndRest?.();
            resolve(rsp);
          } else {
            reject(rsp);
          }
        });
      });
    },
    intl: intl,
  });
};

const handleUpdate = async (
  id: string,
  params: ParamsType,
  ref: React.MutableRefObject<ActionType | undefined>,
) => {
  if (await checkAPI(putAPI(`${API_PREFIX}/scripts/${id}`, params))) {
    ref.current?.reloadAndRest?.();
    return true;
  }
  return false;
};

const handleRun = async (id: string) => {
  await checkAPI(postAPI(`${API_PREFIX}/scripts/${id}/run`, {}));
};

const ScriptCard = () => {
  const intl = getIntl();
  const ref = useRef<ActionType>();

  const columns: ProColumns[] = [
    SearchColumn(intl),
    IDColumn(intl),
    {
      title: intl.get('tables.name'),
      dataIndex: 'name',
      align: 'center',
      hideInSearch: true,
    },
    ...CreatedAtColumn(intl),
    ...UpdatedAtColumn(intl),
    {
      title: intl.get('app.op'),
      valueType: 'option',
      align: 'center',
      className: styles.operation,
      width: 100,
      render: (_, row) => {
        return [
          <TableBtn
            key="run"
            icon={CaretRightOutlined}
            tip={intl.get('pages.script.op.run')}
            perm={UserPerm.PermWrite}
            permName="manage.4adaf7d3-b877-43c3-82bd-da3689dc3920"
            onClick={() => handleRun(row.id)}
          />,
          <TaskEditor
            key="update"
            id={row.id}
            name={row.name}
            trigger={
              <TableBtn
                key="update"
                icon={EditOutlined}
                tip={intl.get('app.op.update')}
              />
            }
            title={intl.get('pages.script.view.title')}
            rollback={<EditOutlined key="update" />}
            onFinish={(params) => handleUpdate(row.id, params, ref)}
          />,
          <TableBtn
            key="delete"
            icon={DeleteOutlined}
            tip={intl.get('app.op.delete')}
            color="#ff4d4f"
            perm={UserPerm.PermWrite}
            permName="manage.4adaf7d3-b877-43c3-82bd-da3689dc3920"
            onClick={() => handleDelete(intl, ref, row.id, row.name)}
          />,
        ];
      },
    },
  ];

  return (
    <Table
      actionRef={ref}
      rowKey="id"
      request={request}
      columns={columns}
      action={[
        <TaskEditor
          key="add"
          trigger={
            <Button key="add" type="primary">
              <PlusOutlined />
              <FormattedMessage id="app.op.add" />
            </Button>
          }
          title={intl.get('pages.script.add.title')}
          rollback={
            <Button key="add" type="primary" disabled>
              <PlusOutlined />
              <FormattedMessage id="app.op.add" />
            </Button>
          }
          onFinish={async (params) => {
            if (await checkAPI(postAPI(`${API_PREFIX}/scripts`, params))) {
              ref.current?.reloadAndRest?.();
              return true;
            }
            return false;
          }}
        />,
      ]}
    />
  );
};

export default ScriptCard;
