import TableOp from '@/common/components/layout/table/opBtn';
import { API_PREFIX, PLUGIN_ID } from '@/config';
import { checkPerm, getAPI, getIntl, UserPerm } from '@/utils';
import { ParamsType, ProFormColumnsType } from '@ant-design/pro-components';
import { useModel } from '@umijs/max';
import _ from 'lodash';
import { ReactElement, useState } from 'react';
import MonacoEditor from 'react-monaco-editor';

export interface TaskEditorProps {
  trigger: JSX.Element;
  rollback: ReactElement<any, any>;
  title: string;
  id?: string;
  name?: string;
  onFinish: (params: ParamsType) => Promise<boolean>;
}

export type TaskEditorHandle = {
  setIsModalOpen: (open: boolean) => void;
};

const TaskEditor: React.FC<TaskEditorProps> = (props: TaskEditorProps) => {
  const intl = getIntl();
  const [initialValues, setInitialValues] = useState<ParamsType>();
  const { access } = useModel('@@qiankunStateFromMaster');
  const onFinish = async (params: ParamsType) => {
    if (props.id)
      _.forEach(params, (v: any, k: any) => {
        if (_.isEqual(initialValues?.[k], v)) delete params[k];
      });
    return props.onFinish(params);
  };

  const columns: ProFormColumnsType[] = [
    {
      title: intl.get('tables.name'),
      dataIndex: 'name',
      fieldProps: {
        maxLength: 32,
      },
      formItemProps: {
        labelAlign: 'left',
        labelCol: {
          span: 3,
        },
        wrapperCol: { span: 12 },
        rules: [{ required: true }],
      },
    },
    {
      dataIndex: 'code',
      renderFormItem: () => {
        return (
          <MonacoEditor
            height={400}
            language="rust"
            theme="vs-dark"
            options={{ readOnly: disable }}
          />
        );
      },
    },
  ];
  const disable = !checkPerm(access, `manage.${PLUGIN_ID}`, UserPerm.PermWrite);

  return (
    <TableOp
      trigger={props.trigger}
      schemaProps={{
        request: async (_params: Record<string, any>, _props: any) => {
          if (props.id) {
            const rsp = await getAPI(`${API_PREFIX}/scripts/${props.id}`);
            setInitialValues(rsp.data);
            return rsp.data;
          }
          return {};
        },
        onFinish: onFinish,
        columns: columns as any,
        initialValues: initialValues,
        disabled: disable,
      }}
      modalProps={{ footer: disable ? null : undefined }}
      disabled={props.id ? false : disable}
      rollback={props.rollback}
      width={1000}
      title={props.title}
      changedSubmit={props.id !== undefined}
    />
  );
};

export default TaskEditor;
